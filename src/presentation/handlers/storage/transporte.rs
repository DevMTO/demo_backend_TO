//! Storage handlers para Transporte (logo)

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
use super::helpers::update_transporte_media;

/// Subir logo de transporte
/// 
/// POST /api/v1/storage/transporte/{transporte_id}/logo
pub async fn upload_transporte_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(transporte_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("📤 Subiendo logo para transporte {}", transporte_id);
    
    // Verificar que el storage está configurado
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            error!("Storage no configurado");
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth_user.user.role == UserRole::Transportes;
    
    // Verificar permisos: SuperAdmin, Admin, o encargado del transporte
    let mut can_upload = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || (is_transporte_user && auth_user.user.id_entidad == Some(transporte_id));
    
    // Si aún no tiene permisos, verificar si es el encargado del transporte
    if !can_upload {
        if let Some(user_persona_id) = auth_user.user.id_persona {
            if let Ok(Some(transporte)) = state.container.transporte_repository.find_by_id(transporte_id).await {
                if transporte.encargado == Some(user_persona_id) {
                    can_upload = true;
                    info!("Usuario {} es encargado de transporte {} (persona_id: {})", 
                        auth_user.user.username, transporte_id, user_persona_id);
                }
            }
        }
    }
    
    if !can_upload {
        warn!("Usuario {} no tiene permisos para subir logo de transporte {}", 
            auth_user.user.username, transporte_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar este transporte".to_string(),
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
        let path = TigrisStorage::generate_transporte_path(transporte_id, "logo", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("Error subiendo a Tigris: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error subiendo archivo: {}", e),
            }))
        })?;
        
        // Actualizar el transporte con la nueva URL del logo
        if let Err(e) = update_transporte_media(
            &state, 
            transporte_id, 
            "logo", 
            &path,
            auth_user.user.id,
        ).await {
            warn!("Error actualizando media de transporte: {}", e);
        }
        
        info!("Logo de transporte subido: {}", url);
        
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
