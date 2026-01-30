//! POST handlers para Conductor

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::CreateConductorRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/conductores - Crear un nuevo conductor
#[instrument(skip(state, auth, request))]
pub async fn create_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateConductorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let created = state.container.conductor_service
        .create_conductor(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Conductor creado: {} (ID: {})", created.nro_brevete, created.id);
    Ok(json_created(created))
}
