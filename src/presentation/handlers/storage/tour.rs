//! Storage handlers para Tour (imagen)

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
    TigrisStorage,
    MAX_FILE_SIZE,
};
use crate::domain::entities::UserRole;
use super::types::{UploadResponse, StorageErrorResponse};
use super::helpers::update_tour_media;

/// Subir imagen de tour
/// 
/// POST /api/v1/storage/tour/{tour_id}/image
pub async fn upload_tour_image(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(tour_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("📤 Subiendo imagen para tour {}", tour_id);
    
    // Verificar que el storage está configurado
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            error!("Storage no configurado");
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar permisos: SuperAdmin, Admin o Agencias pueden subir imágenes de tours
    let can_upload = auth_user.user.role == UserRole::SuperAdmin 
        || auth_user.user.role == UserRole::Admin
        || auth_user.user.role == UserRole::Agencias;
    
    if !can_upload {
        warn!("Usuario {} no tiene permisos para subir imagen de tour {}", 
            auth_user.user.username, tour_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar este tour".to_string(),
        })));
    }
    
    // Verificar que el tour existe
    let _tour = state.container.tour_repository
        .find_by_id(tour_id)
        .await
        .map_err(|e| {
            error!("Error buscando tour: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: "Error verificando tour".to_string(),
            }))
        })?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(StorageErrorResponse {
                success: false,
                error: "Tour no encontrado".to_string(),
            }))
        })?;
    
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
        let path = TigrisStorage::generate_tour_path(tour_id, "image", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("Error subiendo a Tigris: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(StorageErrorResponse {
                success: false,
                error: format!("Error subiendo archivo: {}", e),
            }))
        })?;
        
        // Actualizar el tour con la nueva URL de la imagen
        if let Err(e) = update_tour_media(
            &state, 
            tour_id, 
            "image", 
            &path,
            auth_user.user.id,
        ).await {
            warn!("Error actualizando media de tour: {}", e);
        }
        
        info!("Imagen de tour subida: {}", url);
        
        return Ok(Json(UploadResponse {
            success: true,
            url,
            path,
            message: "Imagen subida correctamente".to_string(),
        }));
    }
    
    Err((StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
        success: false,
        error: "No se recibió ningún archivo".to_string(),
    })))
}
