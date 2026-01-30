//! Handler para actualización de status de FileEntrada

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateRelationStatusRequest, UpdateStatusResponse, FileRelationStatus};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

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
