//! PATCH handlers para User

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;

/// Activar un usuario
/// Solo SuperAdmin puede activar usuarios
#[instrument(skip(state, auth))]
pub async fn activate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede activar usuarios".to_string()));
    }
    
    let (user_dto, _old_active) = state.container.user_service
        .activate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}

/// Desactivar un usuario
/// Solo SuperAdmin puede desactivar usuarios
#[instrument(skip(state, auth))]
pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede desactivar usuarios".to_string()));
    }
    
    let (user_dto, _old_active) = state.container.user_service
        .deactivate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}
