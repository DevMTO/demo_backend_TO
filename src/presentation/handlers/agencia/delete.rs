//! DELETE handlers para Agencias

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// DELETE /api/v1/agencias/:id - Soft delete (desactivar agencia)
#[instrument(skip(state, auth))]
pub async fn delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar agencias".to_string()));
    }
    
    state.container.agencia_service
        .delete_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia desactivada correctamente"))
}

/// DELETE /api/v1/agencias/:id/hard-delete - Hard delete (eliminar permanentemente)
#[instrument(skip(state, auth))]
pub async fn hard_delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin) {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente agencias".to_string()));
    }
    
    state.container.agencia_service
        .hard_delete_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia eliminada permanentemente"))
}
