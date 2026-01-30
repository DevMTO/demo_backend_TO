//! Funciones auxiliares para Storage handlers

use crate::application::dtos::{UpdateAgenciaRequest, UpdateTransporteRequest, UpdateTourRequest};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use serde_json::json;

/// Actualiza el campo media de una agencia
pub async fn update_agencia_media(
    state: &AppState,
    agencia_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    // Obtener agencia actual
    let agencia = state.container.agencia_repository
        .find_by_id(agencia_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Agencia no encontrada".to_string()))?;
    
    // Actualizar media
    let mut media = agencia.get_media().unwrap_or_default();
    match field {
        "logo" => media.logo = Some(path.to_string()),
        "banner" => media.banner = Some(path.to_string()),
        _ => {}
    }
    
    // Construir request de actualización
    let request = UpdateAgenciaRequest {
        nombre: None,
        ruc: None,
        telefono: None,
        correo: None,
        direccion: None,
        paleta_colores: None,
        media: Some(serde_json::to_value(&media).unwrap_or(json!({}))),
        encargado: None,
        is_active: None,
    };
    
    state.container.agencia_service
        .update_agencia(agencia_id, request, updated_by, None)
        .await?;
    
    Ok(())
}

/// Limpia un campo de media de una agencia (lo pone en null)
pub async fn clear_agencia_media(
    state: &AppState,
    agencia_id: i32,
    field: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    let agencia = state.container.agencia_repository
        .find_by_id(agencia_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Agencia no encontrada".to_string()))?;
    
    let mut media = agencia.get_media().unwrap_or_default();
    match field {
        "logo" => media.logo = None,
        "banner" => media.banner = None,
        _ => {}
    }
    
    let request = UpdateAgenciaRequest {
        nombre: None,
        ruc: None,
        telefono: None,
        correo: None,
        direccion: None,
        paleta_colores: None,
        media: Some(serde_json::to_value(&media).unwrap_or(json!({}))),
        encargado: None,
        is_active: None,
    };
    
    state.container.agencia_service
        .update_agencia(agencia_id, request, updated_by, None)
        .await?;
    
    Ok(())
}

/// Actualiza el campo media de un transporte
pub async fn update_transporte_media(
    state: &AppState,
    transporte_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    // Obtener transporte actual
    let transporte = state.container.transporte_repository
        .find_by_id(transporte_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Transporte no encontrado".to_string()))?;
    
    // Parsear media actual o crear nuevo
    let current_media = transporte.media.clone().unwrap_or(json!({}));
    let mut media: serde_json::Value = if current_media.is_string() {
        serde_json::from_str(current_media.as_str().unwrap_or("{}")).unwrap_or(json!({}))
    } else {
        current_media
    };
    
    // Actualizar campo
    media[field] = json!(path);
    
    // Construir request de actualización
    let request = UpdateTransporteRequest {
        nombre: None,
        ruc: None,
        telefono: None,
        correo: None,
        direccion: None,
        encargado: None,
        media: Some(media),
        paleta_colores: None,
        is_active: None,
    };
    
    // Aplicar la actualización usando el repositorio directamente
    let updated = request.apply_to(transporte, Some(updated_by));
    state.container.transporte_repository.update(&updated).await?;
    
    Ok(())
}

/// Actualiza el campo media de un tour
pub async fn update_tour_media(
    state: &AppState,
    tour_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    // Obtener tour actual
    let tour = state.container.tour_repository
        .find_by_id(tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Tour no encontrado".to_string()))?;
    
    // Parsear media actual o crear nuevo
    let current_media = tour.media.clone().unwrap_or(json!({}));
    let mut media: serde_json::Value = if current_media.is_string() {
        serde_json::from_str(current_media.as_str().unwrap_or("{}")).unwrap_or(json!({}))
    } else {
        current_media
    };
    
    // Actualizar campo
    media[field] = json!(path);
    
    // Construir request de actualización
    let request = UpdateTourRequest {
        nombre: None,
        lugar_inicio: None,
        lugar_fin: None,
        detalles: None,
        itinerario: None,
        precio_base: None,
        duracion_dias: None,
        media: Some(media),
        tipo_tour: None,
        horarios: None,
        is_active: None,
        tiene_restaurante: None,
    };
    
    // Aplicar la actualización usando el servicio
    state.container.tour_service
        .update_tour(tour_id, request, updated_by, None)
        .await?;
    
    Ok(())
}
