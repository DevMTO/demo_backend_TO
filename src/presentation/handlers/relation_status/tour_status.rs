//! Handler para actualización de status de FileTour

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateRelationStatusRequest, UpdateStatusResponse, FileRelationStatus, UpdateFileTourHoraRecojoRequest, UpdateFileTourHoraRecojoResponse, UpdateFileTourRecojoRequest, UpdateFileTourRecojoResponse, GeoLocation};
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

    // Usar el servicio para actualizar el status con cascada
    let result = state.container.file_tour_status_service
        .update_status(id, status.as_str())
        .await?;

    Ok(json_ok(UpdateStatusResponse {
        success: true,
        mensaje: result.to_message(),
        old_status: result.old_status,
        new_status: result.new_status,
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

/// Actualiza la información de recojo (hora, lugar y/o geo) de un file_tour
#[instrument(skip(state, _auth))]
pub async fn update_file_tour_recojo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateFileTourRecojoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    // Obtener registro actual
    let current = state.container.file_tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", id)))?;

    let old_hora_recojo = current.hora_recojo;
    let old_lugar_recojo = current.lugar_recojo.clone();
    let old_geo_recojo = current.geo_recojo.clone();

    // Convertir GeoLocation a serde_json::Value si existe
    let geo_recojo_value = request.geo_recojo.as_ref()
        .map(|geo| serde_json::to_value(geo).unwrap_or(serde_json::Value::Null));

    // Actualizar recojo
    state.container.file_tour_repository
        .update_recojo(id, request.hora_recojo, request.lugar_recojo.clone(), geo_recojo_value)
        .await?;

    // Construir mensaje descriptivo
    let mut cambios = Vec::new();

    match (&old_hora_recojo, &request.hora_recojo) {
        (Some(old), Some(new)) if old != new => cambios.push(format!("hora: '{}' → '{}'", old, new)),
        (Some(old), None) => cambios.push(format!("hora '{}' eliminada", old)),
        (None, Some(new)) => cambios.push(format!("hora establecida a '{}'", new)),
        _ => {}
    }

    match (&old_lugar_recojo, &request.lugar_recojo) {
        (Some(old), Some(new)) if old != new => cambios.push(format!("lugar: '{}' → '{}'", old, new)),
        (Some(old), None) => cambios.push(format!("lugar '{}' eliminado", old)),
        (None, Some(new)) => cambios.push(format!("lugar establecido a '{}'", new)),
        _ => {}
    }

    match (&old_geo_recojo, &request.geo_recojo) {
        (Some(_), Some(_)) => cambios.push("geolocalización actualizada".to_string()),
        (Some(_), None) => cambios.push("geolocalización eliminada".to_string()),
        (None, Some(_)) => cambios.push("geolocalización establecida".to_string()),
        _ => {}
    }

    let mensaje = if cambios.is_empty() {
        "Sin cambios en información de recojo".to_string()
    } else {
        format!("Recojo actualizado: {}", cambios.join(", "))
    };

    // Convertir old_geo_recojo de JsonValue a GeoLocation para la respuesta
    let old_geo_recojo_dto = old_geo_recojo
        .and_then(|v| serde_json::from_value::<GeoLocation>(v).ok());

    Ok(json_ok(UpdateFileTourRecojoResponse {
        success: true,
        mensaje,
        old_hora_recojo,
        new_hora_recojo: request.hora_recojo,
        old_lugar_recojo,
        new_lugar_recojo: request.lugar_recojo,
        old_geo_recojo: old_geo_recojo_dto,
        new_geo_recojo: request.geo_recojo,
    }))
}
