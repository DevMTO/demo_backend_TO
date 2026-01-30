//! PUT handlers para Tour

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::UpdateTourRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Actualizar tour existente
#[instrument(skip(state, auth, request))]
pub async fn update_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.tour_service
        .update_tour(
            id,
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Tour actualizado: {} (ID: {})", response.nombre, response.id);
    Ok(json_ok(response))
}
