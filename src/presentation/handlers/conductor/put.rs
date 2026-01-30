//! PUT handlers para Conductor

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::UpdateConductorRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/v1/conductores/:id - Actualizar un conductor
#[instrument(skip(state, auth, request))]
pub async fn update_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateConductorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let updated = state.container.conductor_service
        .update_conductor(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Conductor actualizado: {} (ID: {})", updated.nro_brevete, id);
    Ok(json_ok(updated))
}
