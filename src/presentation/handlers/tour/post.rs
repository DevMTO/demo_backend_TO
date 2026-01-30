//! POST handlers para Tour

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::CreateTourRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// Crear nuevo tour
#[instrument(skip(state, auth, request))]
pub async fn create_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.tour_service
        .create_tour(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Tour creado: {} (ID: {})", response.nombre, response.id);
    Ok(json_created(response))
}
