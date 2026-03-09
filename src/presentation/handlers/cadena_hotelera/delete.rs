//! DELETE handlers para Cadenas Hoteleras

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

/// DELETE /api/v1/cadenas-hoteleras/:id - Soft delete (desactivar)
#[instrument(skip(state, auth))]
pub async fn delete_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar cadenas hoteleras".to_string()));
    }
    
    state.container.cadena_hotelera_service
        .delete_cadena(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Cadena hotelera desactivada correctamente"))
}

/// DELETE /api/v1/cadenas-hoteleras/:id/hard-delete - Hard delete
#[instrument(skip(state, auth))]
pub async fn hard_delete_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin) {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente cadenas hoteleras".to_string()));
    }
    
    state.container.cadena_hotelera_service
        .hard_delete_cadena(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Cadena hotelera eliminada permanentemente"))
}
