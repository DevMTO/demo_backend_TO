//! Storage handlers para Agencia (logo y banner)

use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    Json,
};
use tracing::{info, error, warn};

use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::infrastructure::storage::{
    validate_content_type, 
    extension_from_content_type, 
    infer_content_type_from_filename,
    TigrisStorage,
    MAX_FILE_SIZE,
};
use crate::domain::entities::UserRole;
use super::types::{UploadResponse, StorageErrorResponse, StorageDeleteResponse};
use super::helpers::{update_agencia_media, clear_agencia_media};

/// Subir logo de agencia
/// 
/// POST /api/v1/storage/agencia/{agencia_id}/logo
pub async fn upload_agencia_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(agencia_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("📤 Subiendo logo para agencia {}", agencia_id);
    
    // Verificar que el storage está configurado
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            error!("Storage no configurado");
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth_user.user.role == UserRole::Agencias || auth_user.user.role == UserRole::AgenciasGerente;
    
    // Verificar permisos: SuperAdmin, Admin, o encargado de la agencia
    let mut can_upload = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || (is_agencia_user && auth_user.user.id_entidad == Some(agencia_id));
    
    // Si aún no tiene permisos, verificar si es el encargado de la agencia
    if !can_upload {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            // Buscar la agencia para ver si este usuario es el encargado
            if let Ok(Some(agencia)) = state.container.agencia_repository.find_by_id(agencia_id).await {
                if agencia.encargado == Some(user_persona_id) {
                    can_upload = true;
                    info!("Usuario {} es encargado de agencia {} (persona_id: {})", 
                        auth_user.user.username, agencia_id, user_persona_id);
                }
            }
        }
    }
    
    if !can_upload {
        warn!("Usuario {} no tiene permisos para subir logo de agencia {}", 
            auth_user.user.username, agencia_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta agencia".to_string(),
        })));
    }
    
    // Procesar archivo del multipart
    if let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Error procesando multipart: {}", e);
        (StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
            success: false,
            error: format!("Error procesando archivo: {}", e),
        }))
    })? {
        let content_type = field.content_type()
            .map(|ct| ct.to_string())
            .or_else(|| {
                field.file_name().and_then(|name| infer_content_type_from_filename(name))
            })
            .unwrap_or_default();
        
        // Validar tipo de archivo
        if let Err(e) = validate_content_type(&content_type) {
            return Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: e,
            })));
        }
        
        // Leer bytes del archivo
        let data = field.bytes().await.map_err(|e| {
            error!("Error leyendo archivo: {}", e);
            (StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: format!("Error leyendo archivo: {}", e),
            }))
        })?;
        
        // Validar tamaño
        if data.len() > MAX_FILE_SIZE {
            return Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: format!("Archivo muy grande. Máximo: {} MB", MAX_FILE_SIZE / 1024 / 1024),
            })));
        }
        
        // Generar path y subir
        let extension = extension_from_content_type(&content_type);
        let path = TigrisStorage::generate_agencia_path(agencia_id, "logo", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("Error subiendo a Tigris: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error subiendo archivo: {}", e),
            }))
        })?;
        
        // Actualizar la agencia con la nueva URL del logo
        if let Err(e) = update_agencia_media(
            &state, 
            agencia_id, 
            "logo", 
            &path,
            auth_user.user.id,
        ).await {
            warn!("Error actualizando media de agencia: {}", e);
        }
        
        info!("Logo subido: {}", url);
        
        return Ok(Json(UploadResponse {
            success: true,
            url,
            path,
            message: "Logo subido correctamente".to_string(),
        }));
    }
    
    Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
        success: false,
        error: "No se recibió ningún archivo".to_string(),
    })))
}

/// Subir banner de agencia
/// 
/// POST /api/v1/storage/agencia/{agencia_id}/banner
pub async fn upload_agencia_banner(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(agencia_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("📤 Subiendo banner para agencia {}", agencia_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth_user.user.role == UserRole::Agencias || auth_user.user.role == UserRole::AgenciasGerente;
    
    // Verificar permisos
    let mut can_upload = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || (is_agencia_user && auth_user.user.id_entidad == Some(agencia_id));
    
    // Si aún no tiene permisos, verificar si es el encargado de la agencia
    if !can_upload {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            if let Ok(Some(agencia)) = state.container.agencia_repository.find_by_id(agencia_id).await {
                if agencia.encargado == Some(user_persona_id) {
                    can_upload = true;
                    info!("Usuario {} es encargado de agencia {} (persona_id: {})", 
                        auth_user.user.username, agencia_id, user_persona_id);
                }
            }
        }
    }
    
    if !can_upload {
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta agencia".to_string(),
        })));
    }
    
    if let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
            success: false,
            error: format!("Error procesando archivo: {}", e),
        }))
    })? {
        let content_type = field.content_type()
            .map(|ct| ct.to_string())
            .or_else(|| {
                field.file_name().and_then(|name| infer_content_type_from_filename(name))
            })
            .unwrap_or_default();
        
        if let Err(e) = validate_content_type(&content_type) {
            return Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: e,
            })));
        }
        
        let data = field.bytes().await.map_err(|e| {
            (StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: format!("Error leyendo archivo: {}", e),
            }))
        })?;
        
        if data.len() > MAX_FILE_SIZE {
            return Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: format!("Archivo muy grande. Máximo: {} MB", MAX_FILE_SIZE / 1024 / 1024),
            })));
        }
        
        let extension = extension_from_content_type(&content_type);
        let path = TigrisStorage::generate_agencia_path(agencia_id, "banner", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error subiendo archivo: {}", e),
            }))
        })?;
        
        if let Err(e) = update_agencia_media(&state, agencia_id, "banner", &path, auth_user.user.id).await {
            warn!("Error actualizando media de agencia: {}", e);
        }
        
        return Ok(Json(UploadResponse {
            success: true,
            url,
            path,
            message: "Banner subido correctamente".to_string(),
        }));
    }
    
    Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
        success: false,
        error: "No se recibió ningún archivo".to_string(),
    })))
}

/// Eliminar logo de agencia
/// 
/// DELETE /api/v1/storage/agencia/{agencia_id}/logo
pub async fn delete_agencia_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(agencia_id): Path<i32>,
) -> Result<Json<StorageDeleteResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("[DELETE] Eliminando logo para agencia {}", agencia_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar permisos
    let is_agencia_user = auth_user.user.role == UserRole::Agencias || auth_user.user.role == UserRole::AgenciasGerente;
    
    let mut can_delete = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || (is_agencia_user && auth_user.user.id_entidad == Some(agencia_id));
    
    if !can_delete {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            if let Ok(Some(agencia)) = state.container.agencia_repository.find_by_id(agencia_id).await {
                if agencia.encargado == Some(user_persona_id) {
                    can_delete = true;
                }
            }
        }
    }
    
    if !can_delete {
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta agencia".to_string(),
        })));
    }
    
    // Obtener la agencia para ver el path actual del logo
    let agencia = state.container.agencia_repository
        .find_by_id(agencia_id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error obteniendo agencia: {}", e),
            }))
        })?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(StorageErrorResponse {
                success: false,
                error: "Agencia no encontrada".to_string(),
            }))
        })?;
    
    let media = agencia.get_media().unwrap_or_default();
    
    if let Some(logo_path) = &media.logo {
        // Eliminar archivo de Tigris
        if let Err(e) = storage.delete(logo_path).await {
            warn!("Error eliminando archivo de Tigris: {}", e);
            // Continuamos para limpiar la BD aunque falle Tigris
        }
    }
    
    // Actualizar media quitando el logo
    if let Err(e) = clear_agencia_media(&state, agencia_id, "logo", auth_user.user.id).await {
        warn!("Error limpiando media de agencia: {}", e);
    }
    
    info!("Logo eliminado para agencia {}", agencia_id);
    
    Ok(Json(StorageDeleteResponse {
        success: true,
        message: "Logo eliminado correctamente".to_string(),
    }))
}

/// Eliminar banner de agencia
/// 
/// DELETE /api/v1/storage/agencia/{agencia_id}/banner
pub async fn delete_agencia_banner(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(agencia_id): Path<i32>,
) -> Result<Json<StorageDeleteResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("[DELETE] Eliminando banner para agencia {}", agencia_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar permisos
    let is_agencia_user = auth_user.user.role == UserRole::Agencias || auth_user.user.role == UserRole::AgenciasGerente;
    
    let mut can_delete = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || (is_agencia_user && auth_user.user.id_entidad == Some(agencia_id));
    
    if !can_delete {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            if let Ok(Some(agencia)) = state.container.agencia_repository.find_by_id(agencia_id).await {
                if agencia.encargado == Some(user_persona_id) {
                    can_delete = true;
                }
            }
        }
    }
    
    if !can_delete {
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta agencia".to_string(),
        })));
    }
    
    // Obtener la agencia para ver el path actual del banner
    let agencia = state.container.agencia_repository
        .find_by_id(agencia_id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error obteniendo agencia: {}", e),
            }))
        })?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(StorageErrorResponse {
                success: false,
                error: "Agencia no encontrada".to_string(),
            }))
        })?;
    
    let media = agencia.get_media().unwrap_or_default();
    
    if let Some(banner_path) = &media.banner {
        // Eliminar archivo de Tigris
        if let Err(e) = storage.delete(banner_path).await {
            warn!("Error eliminando archivo de Tigris: {}", e);
        }
    }
    
    // Actualizar media quitando el banner
    if let Err(e) = clear_agencia_media(&state, agencia_id, "banner", auth_user.user.id).await {
        warn!("Error limpiando media de agencia: {}", e);
    }
    
    info!("Banner eliminado para agencia {}", agencia_id);
    
    Ok(Json(StorageDeleteResponse {
        success: true,
        message: "Banner eliminado correctamente".to_string(),
    }))
}
