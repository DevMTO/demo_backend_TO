//! POST handlers para Vehiculo

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_created;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::CreateVehiculoRequest;

/// Crear un nuevo vehículo
#[instrument(skip(state, auth, request))]
pub async fn create_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateVehiculoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let vehiculo = state.container.vehiculo_service
        .create_vehiculo(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(vehiculo))
}
