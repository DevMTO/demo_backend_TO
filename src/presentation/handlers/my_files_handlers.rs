//! Handlers para que los usuarios vean sus files asignados según su rol
//! - Guías ven sus files asignados
//! - Conductores ven sus files con vehículos asignados
//! - Restaurantes ven sus files asignados
//! - Confirmación de asignaciones

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::{error, info, instrument};
use validator::Validate;

use crate::application::dtos::{
    ConfirmFileGuiaAssignmentRequest,
    ConfirmFileVehiculoAssignmentRequest,
};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::json_ok;

/// GET /api/v1/my-files/guia - Obtiene los files asignados al guía autenticado
/// Requiere que el usuario tenga id_persona y sea guía
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_guia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    // Verificar que el usuario es un guía
    if auth.user.role != UserRole::Guias {
        return Err(ApplicationError::Forbidden("Solo los guías pueden acceder a este endpoint".to_string()));
    }
    
    info!("📋 Consultando files para guía con id_persona: {}", id_persona);
    
    let files = match state.container.my_files_service.get_my_files_as_guia(id_persona).await {
        Ok(files) => {
            info!("📋 Encontrados {} files para guía id_persona: {}", files.len(), id_persona);
            files
        },
        Err(e) => {
            error!("❌ Error consultando files para guía id_persona {}: {:?}", id_persona, e);
            return Err(e);
        }
    };
    
    Ok(json_ok(files))
}

/// GET /api/v1/my-files/conductor - Obtiene los files asignados al conductor autenticado
/// Requiere que el usuario tenga id_persona y sea conductor
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    // Verificar que el usuario es un conductor
    if auth.user.role != UserRole::Conductores {
        return Err(ApplicationError::Forbidden("Solo los conductores pueden acceder a este endpoint".to_string()));
    }
    
    let files = state.container.my_files_service.get_my_files_as_conductor(id_persona).await?;
    Ok(json_ok(files))
}

/// GET /api/v1/my-files/restaurante - Obtiene los files asignados al restaurante autenticado
/// Requiere que el usuario tenga role restaurantes y id_entidad configurado
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es un restaurante
    if auth.user.role != UserRole::Restaurantes {
        return Err(ApplicationError::Forbidden("Solo los restaurantes pueden acceder a este endpoint".to_string()));
    }
    
    // Obtener el id_restaurante desde id_entidad del usuario
    let id_restaurante = auth.user.id_entidad
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene restaurante asociado (id_entidad)".to_string()))?;
    
    let files = state.container.my_files_service.get_my_files_as_restaurante(id_restaurante).await?;
    Ok(json_ok(files))
}

// ==================== CONFIRMACIÓN DE ASIGNACIONES ====================

/// POST /api/v1/my-files/guia/{file_id}/confirm
/// Un guía confirma (acepta o rechaza) su asignación a un file específico
#[instrument(skip(state, auth, payload))]
pub async fn confirm_guia_assignment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
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
    
    info!("🔔 Guía (persona: {}) confirmando asignación al file {}: aceptar={}", 
          id_persona, file_id, payload.aceptar);
    
    let response = state.container.my_files_service
        .confirm_guia_assignment(id_persona, file_id, payload.aceptar, payload.motivo_rechazo)
        .await?;
    
    Ok(json_ok(response))
}

/// POST /api/v1/my-files/conductor/{file_id}/confirm
/// Un conductor confirma (acepta o rechaza) su asignación a un file específico
#[instrument(skip(state, auth, payload))]
pub async fn confirm_conductor_assignment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
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
    
    info!("🔔 Conductor (persona: {}) confirmando asignación al file {}: aceptar={}", 
          id_persona, file_id, payload.aceptar);
    
    let response = state.container.my_files_service
        .confirm_conductor_assignment(id_persona, file_id, payload.aceptar, payload.motivo_rechazo)
        .await?;
    
    Ok(json_ok(response))
}
