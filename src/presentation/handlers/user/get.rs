//! GET handlers para User

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::instrument;

use crate::application::ports::UserListScope;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, create_paginated_response};
use crate::presentation::extractors::AuthUser;

#[derive(Debug, Deserialize)]
pub struct ListUsersParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default)]
    pub is_demo: Option<bool>,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

/// Listar usuarios con paginación.
/// SuperAdmin y Admin ven todos; gerentes solo ven usuarios en su ámbito.
#[instrument(skip(state, auth))]
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListUsersParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let scope = match auth.user.role {
        UserRole::AgenciasGerente => match auth.user.id_entidad {
            Some(id) => UserListScope::AgenciaScope { id_entidad: id },
            None => UserListScope::All,
        },
        UserRole::HotelesGerente => match auth.user.id_entidad {
            Some(id) => UserListScope::HotelCadenaScope { id_cadena: id },
            None => UserListScope::All,
        },
        _ => UserListScope::All,
    };

    let (users, total) = state.container.user_service
        .list_users(params.page, params.page_size, params.is_demo, &scope)
        .await?;

    let page_size = params.page_size.clamp(1, 10000);
    let response = create_paginated_response(users, total, params.page, page_size);

    Ok(json_ok(response))
}

/// Obtener un usuario por ID
#[instrument(skip(state, _auth))]
pub async fn get_user(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let user = state.container.user_service
        .get_user(id)
        .await?;

    Ok(json_ok(user))
}
