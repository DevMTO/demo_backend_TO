//! Storage handlers para Cadena Hotelera (logo)

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
use super::types::{UploadResponse, StorageErrorResponse};
use super::helpers::update_cadena_media;

/// Subir logo de cadena hotelera
/// 
/// POST /api/v1/storage/cadena/{cadena_id}/logo
pub async fn upload_cadena_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(cadena_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("📤 Subiendo logo para cadena hotelera {}", cadena_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            error!("Storage no configurado");
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Permisos: SuperAdmin, Admin, o usuario hotel con id_entidad de esta cadena
    let is_hotel_user = matches!(auth_user.user.role, UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena);
    
    let mut can_upload = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin;
    
    // Verificar si es encargado de la cadena
    if !can_upload {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            if let Ok(Some(cadena)) = state.container.cadena_hotelera_repository.find_by_id(cadena_id).await {
                if cadena.encargado == Some(user_persona_id) {
                    can_upload = true;
                }
            }
        }
    }
    
    // Verificar si es usuario de hotel perteneciente a esta cadena
    if !can_upload && is_hotel_user {
        if let Some(hotel_id) = auth_user.user.id_entidad {
            if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(hotel_id).await {
                if hotel.id_cadena == cadena_id {
                    can_upload = true;
                }
            }
        }
    }
    
    if !can_upload {
        warn!("Usuario {} no tiene permisos para subir logo de cadena {}", 
            auth_user.user.username, cadena_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta cadena".to_string(),
        })));
    }
    
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
        
        if let Err(e) = validate_content_type(&content_type) {
            return Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
                success: false,
                error: e,
            })));
        }
        
        let data = field.bytes().await.map_err(|e| {
            error!("Error leyendo archivo: {}", e);
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
        let path = TigrisStorage::generate_cadena_path(cadena_id, "logo", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("Error subiendo a Tigris: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error subiendo archivo: {}", e),
            }))
        })?;
        
        if let Err(e) = update_cadena_media(
            &state, 
            cadena_id, 
            "logo", 
            &path,
            auth_user.user.id,
        ).await {
            warn!("Error actualizando media de cadena: {}", e);
        }
        
        info!("Logo de cadena subido: {}", url);
        
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

/// Eliminar logo de cadena hotelera
/// 
/// DELETE /api/v1/storage/cadena/{cadena_id}/logo
pub async fn delete_cadena_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(cadena_id): Path<i32>,
) -> Result<Json<super::types::StorageDeleteResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("🗑️ Eliminando logo de cadena hotelera {}", cadena_id);
    
    let can_delete = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin;
    
    if !can_delete {
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para eliminar el logo de esta cadena".to_string(),
        })));
    }
    
    // Clear the logo path in the entity media
    if let Err(e) = super::helpers::clear_cadena_media(
        &state,
        cadena_id,
        "logo",
        auth_user.user.id,
    ).await {
        warn!("Error limpiando media de cadena: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
            success: false,
            error: format!("Error: {}", e),
        })));
    }
    
    Ok(Json(super::types::StorageDeleteResponse {
        success: true,
        message: "Logo eliminado correctamente".to_string(),
    }))
}
