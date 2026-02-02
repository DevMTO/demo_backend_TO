//! Handler para actualización de status de FileTour

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateRelationStatusRequest, UpdateStatusResponse, FileRelationStatus, UpdateFileTourHoraRecojoRequest, UpdateFileTourHoraRecojoResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

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

/// Actualiza la hora de recojo de un file_tour
#[instrument(skip(state, _auth))]
pub async fn update_file_tour_hora_recojo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateFileTourHoraRecojoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Obtener registro actual
    let current = state.container.file_tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", id)))?;
    
    let old_hora_recojo = current.hora_recojo;
    
    // Actualizar hora_recojo
    state.container.file_tour_repository
        .update_hora_recojo(id, request.hora_recojo)
        .await?;
    
    let mensaje = match (&old_hora_recojo, &request.hora_recojo) {
        (Some(old), Some(new)) => format!("Hora de recojo actualizada de '{}' a '{}'", old, new),
        (Some(old), None) => format!("Hora de recojo '{}' eliminada", old),
        (None, Some(new)) => format!("Hora de recojo establecida a '{}'", new),
        (None, None) => "Sin cambios en hora de recojo".to_string(),
    };
    
    Ok(json_ok(UpdateFileTourHoraRecojoResponse {
        success: true,
        mensaje,
        old_hora_recojo,
        new_hora_recojo: request.hora_recojo,
    }))
}
