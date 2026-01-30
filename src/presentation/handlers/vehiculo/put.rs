//! PUT handlers para Vehiculo

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::UpdateVehiculoRequest;

/// Actualizar un vehículo existente
#[instrument(skip(state, auth, request))]
pub async fn update_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateVehiculoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let vehiculo = state.container.vehiculo_service
        .update_vehiculo(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(vehiculo))
}
