//! POST handlers para Persona

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::CreatePersonaRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// Crear nueva persona
#[instrument(skip(state, auth, request))]
pub async fn create_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreatePersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    info!("Creando persona: {} {}", request.nombre, request.apellidos);
    
    let response = state.container.persona_service
        .create_persona(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona creada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_created(response))
}
