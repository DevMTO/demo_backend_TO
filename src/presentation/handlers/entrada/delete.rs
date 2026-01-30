//! DELETE handlers para Entrada

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_message, json_deleted};

#[instrument(skip(state, auth))]
pub async fn delete_entrada(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_service.deactivate_entrada(id, auth.user.id, &auth.user.username).await?;
    Ok(json_message("Entrada desactivada"))
}

/// Eliminación permanente de entrada (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_entrada(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente entradas".to_string()));
    }
    state.container.entrada_service.hard_delete_entrada(id, auth.user.id, &auth.user.username).await?;
    Ok(json_deleted())
}
