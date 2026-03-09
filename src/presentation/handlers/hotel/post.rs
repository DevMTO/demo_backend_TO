//! POST handlers para Hoteles

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreateHotelRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/hoteles - Crear nuevo hotel
#[instrument(skip(state, auth, request))]
pub async fn create_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateHotelRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear hoteles".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.hotel_service
        .create_hotel(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(response))
}
