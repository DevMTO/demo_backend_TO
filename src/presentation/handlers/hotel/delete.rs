//! DELETE handlers para Hoteles

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

/// DELETE /api/v1/hoteles/:id - Soft delete (desactivar hotel)
#[instrument(skip(state, auth))]
pub async fn delete_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar hoteles".to_string()));
    }
    
    state.container.hotel_service
        .delete_hotel(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Hotel desactivado correctamente"))
}

/// DELETE /api/v1/hoteles/:id/hard-delete - Hard delete
#[instrument(skip(state, auth))]
pub async fn hard_delete_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin) {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente hoteles".to_string()));
    }
    
    state.container.hotel_service
        .hard_delete_hotel(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Hotel eliminado permanentemente"))
}
