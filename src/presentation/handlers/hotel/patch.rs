//! PATCH handlers para Hoteles

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

/// PATCH /api/v1/hoteles/:id/restore - Restaurar hotel desactivado
#[instrument(skip(state, auth))]
pub async fn restore_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar hoteles".to_string()));
    }
    
    let _response = state.container.hotel_service
        .restore_hotel(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Hotel restaurado correctamente"))
}
