//! POST handlers para My Files
//! Endpoints para confirmación de asignaciones

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{
    ConfirmFileGuiaAssignmentRequest,
    ConfirmFileVehiculoAssignmentRequest,
};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// POST /api/v1/my-files/guia/{file_tour_id}/confirm
/// Un guía confirma (acepta o rechaza) su asignación a un file_tour específico
#[instrument(skip(state, auth, payload))]
pub async fn confirm_guia_assignment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_tour_id): Path<i32>,
    Json(payload): Json<ConfirmFileGuiaAssignmentRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    payload.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el usuario tiene id_persona y es guía
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    if auth.user.role != UserRole::Guias {
        return Err(ApplicationError::Forbidden("Solo los guías pueden confirmar asignaciones de guía".to_string()));
    }
    
    // Si rechaza, el motivo es obligatorio
    if !payload.aceptar && payload.motivo_rechazo.as_ref().map(|m| m.trim().is_empty()).unwrap_or(true) {
        return Err(ApplicationError::Validation("Debe proporcionar un motivo para rechazar la asignación".to_string()));
    }
    
    info!("Guía (persona: {}) confirmando asignación al file_tour {}: aceptar={}", 
          id_persona, file_tour_id, payload.aceptar);
    
    let response = state.container.my_files_service
        .confirm_guia_assignment(id_persona, file_tour_id, payload.aceptar, payload.motivo_rechazo)
        .await?;
    
    Ok(json_ok(response))
}

/// POST /api/v1/my-files/conductor/{file_tour_id}/confirm
/// Un conductor confirma (acepta o rechaza) su asignación a un file_tour específico
#[instrument(skip(state, auth, payload))]
pub async fn confirm_conductor_assignment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_tour_id): Path<i32>,
    Json(payload): Json<ConfirmFileVehiculoAssignmentRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    payload.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el usuario tiene id_persona y es conductor
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    if auth.user.role != UserRole::Conductores {
        return Err(ApplicationError::Forbidden("Solo los conductores pueden confirmar asignaciones de conductor".to_string()));
    }
    
    // Si rechaza, el motivo es obligatorio
    if !payload.aceptar && payload.motivo_rechazo.as_ref().map(|m| m.trim().is_empty()).unwrap_or(true) {
        return Err(ApplicationError::Validation("Debe proporcionar un motivo para rechazar la asignación".to_string()));
    }
    
    info!("Conductor (persona: {}) confirmando asignación al file_tour {}: aceptar={}", 
          id_persona, file_tour_id, payload.aceptar);
    
    let response = state.container.my_files_service
        .confirm_conductor_assignment(id_persona, file_tour_id, payload.aceptar, payload.motivo_rechazo)
        .await?;
    
    Ok(json_ok(response))
}
