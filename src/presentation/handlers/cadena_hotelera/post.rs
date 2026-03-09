//! POST handlers para Cadenas Hoteleras

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreateCadenaHoteleraRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/cadenas-hoteleras - Crear nueva cadena hotelera
#[instrument(skip(state, auth, request))]
pub async fn create_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateCadenaHoteleraRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear cadenas hoteleras".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.cadena_hotelera_service
        .create_cadena(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(response))
}
