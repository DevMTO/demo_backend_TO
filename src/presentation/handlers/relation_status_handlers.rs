//! Handlers para actualización de status de file_relations
//! Separados del archivo principal para mantener el código organizado

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateRelationStatusRequest, UpdateStatusResponse, FileRelationStatus};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::json_ok;

// ==================== FILE ENTRADA STATUS ====================

/// Actualiza el status de una file_entrada
#[instrument(skip(state, _auth))]
pub async fn update_file_entrada_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    if !status.is_valid_for_other() {
        return Err(ApplicationError::Validation("El status 'pendiente' solo es válido para guías".to_string()));
    }
    
    // Obtener registro actual
    let current = state.container.file_entrada_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileEntrada {} no encontrada", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_entrada_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de entrada actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}

// ==================== FILE GUIA STATUS ====================

/// Actualiza el status de una file_guia (permite 'pendiente')
#[instrument(skip(state, _auth))]
pub async fn update_file_guia_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status (guías permite todos los estados incluyendo pendiente)
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    // Obtener registro actual
    let current = state.container.file_guia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileGuia {} no encontrada", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_guia_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de guía actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}

// ==================== FILE PASAJERO STATUS ====================

/// Actualiza el status de un file_pasajero
#[instrument(skip(state, _auth))]
pub async fn update_file_pasajero_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    if !status.is_valid_for_other() {
        return Err(ApplicationError::Validation("El status 'pendiente' solo es válido para guías".to_string()));
    }
    
    // Obtener registro actual
    let current = state.container.file_pasajero_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FilePasajero {} no encontrado", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_pasajero_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de pasajero actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}

// ==================== FILE RESTAURANTE STATUS ====================

/// Actualiza el status de una file_restaurante
#[instrument(skip(state, _auth))]
pub async fn update_file_restaurante_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    if !status.is_valid_for_other() {
        return Err(ApplicationError::Validation("El status 'pendiente' solo es válido para guías".to_string()));
    }
    
    // Obtener registro actual
    let current = state.container.file_restaurante_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileRestaurante {} no encontrado", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_restaurante_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de restaurante actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}

// ==================== FILE VEHICULO STATUS ====================

/// Actualiza el status de un file_vehiculo (la relación, no el vehículo en sí)
#[instrument(skip(state, _auth))]
pub async fn update_file_vehiculo_status_relation(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    if !status.is_valid_for_other() {
        return Err(ApplicationError::Validation("El status 'pendiente' solo es válido para guías".to_string()));
    }
    
    // Obtener registro actual
    let current = state.container.file_vehiculo_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileVehiculo {} no encontrado", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_vehiculo_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de vehículo actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}

// ==================== FILE TOUR STATUS ====================

/// Actualiza el status de un file_tour
#[instrument(skip(state, _auth))]
pub async fn update_file_tour_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRelationStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Validar status
    let status = FileRelationStatus::from_str(&request.status)
        .map_err(|e| ApplicationError::Validation(e))?;
    
    if !status.is_valid_for_other() {
        return Err(ApplicationError::Validation("El status 'pendiente' solo es válido para guías".to_string()));
    }
    
    // Obtener registro actual
    let current = state.container.file_tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", id)))?;
    
    let old_status = current.status.clone();
    
    // Actualizar status
    state.container.file_tour_repository
        .update_status(id, status.as_str())
        .await?;
    
    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: format!("Status de tour actualizado de '{}' a '{}'", old_status, status.as_str()),
        old_status,
        new_status: status.as_str().to_string(),
    }))
}
