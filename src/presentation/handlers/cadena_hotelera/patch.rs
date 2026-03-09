//! PATCH handlers para Cadenas Hoteleras

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

/// PATCH /api/v1/cadenas-hoteleras/:id/restore - Restaurar cadena hotelera desactivada
#[instrument(skip(state, auth))]
pub async fn restore_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar cadenas hoteleras".to_string()));
    }
    
    let _response = state.container.cadena_hotelera_service
        .restore_cadena(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Cadena hotelera restaurada correctamente"))
}
