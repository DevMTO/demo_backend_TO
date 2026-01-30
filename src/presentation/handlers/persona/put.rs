//! PUT handlers para Persona

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, info, instrument};
use validator::Validate;

use crate::application::dtos::UpdatePersonaRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Actualizar persona existente
#[instrument(skip(state, auth, request))]
pub async fn update_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdatePersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    debug!("Actualizando persona ID: {}", id);
    
    let response = state.container.persona_service
        .update_persona(
            id,
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona actualizada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_ok(response))
}
