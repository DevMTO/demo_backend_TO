use axum::{
    extract::{Path, State, Multipart},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::{info, error, warn};

use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::infrastructure::storage::{
    validate_content_type, 
    extension_from_content_type, 
    TigrisStorage,
    MAX_FILE_SIZE,
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;

/// Respuesta de subida de archivo
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub url: String,
    pub path: String,
    pub message: String,
}

/// Respuesta de error
#[derive(Debug, Serialize)]
pub struct StorageErrorResponse {
    pub success: bool,
    pub error: String,
}

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
            error!("❌ Storage no configurado");
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar si el usuario está relacionado con agencias (nombre contiene "agencia")
    let is_agencia_user = auth_user.user.nombre_entidad
        .as_ref()
        .map(|n| n.to_lowercase().contains("agencia"))
        .unwrap_or(false);
    
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
                    info!("✅ Usuario {} es encargado de agencia {} (persona_id: {})", 
                        auth_user.user.username, agencia_id, user_persona_id);
                }
            }
        }
    }
    
    if !can_upload {
        warn!("⚠️ Usuario {} no tiene permisos para subir logo de agencia {}", 
            auth_user.user.username, agencia_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar esta agencia".to_string(),
        })));
    }
    
    // Procesar archivo del multipart
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("❌ Error procesando multipart: {}", e);
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
            error!("❌ Error leyendo archivo: {}", e);
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
            error!("❌ Error subiendo a Tigris: {}", e);
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
            warn!("⚠️ Error actualizando media de agencia: {}", e);
        }
        
        info!("✅ Logo subido: {}", url);
        
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
    
    // Verificar si el usuario está relacionado con agencias (nombre contiene "agencia")
    let is_agencia_user = auth_user.user.nombre_entidad
        .as_ref()
        .map(|n| n.to_lowercase().contains("agencia"))
        .unwrap_or(false);
    
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
                    info!("✅ Usuario {} es encargado de agencia {} (persona_id: {})", 
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
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(StorageErrorResponse {
            success: false,
            error: format!("Error procesando archivo: {}", e),
        }))
    })? {
        let content_type = field.content_type()
            .map(|ct| ct.to_string())
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
            warn!("⚠️ Error actualizando media de agencia: {}", e);
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

/// Proxy para servir archivos de Tigris
/// 
/// GET /api/v1/storage/proxy/*path
/// 
/// Este endpoint sirve como proxy para archivos almacenados en Tigris,
/// permitiendo control de acceso y caché.
/// Requiere autenticación para seguridad.
pub async fn proxy_file(
    State(state): State<AppState>,
    _auth_user: AuthUser, // Requiere sesión activa
    Path(file_path): Path<String>,
) -> Response {
    let storage = match state.container.tigris_storage.as_ref() {
        Some(s) => s,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Storage no disponible").into_response();
        }
    };
    
    match storage.get(&file_path).await {
        Ok((data, content_type)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE, 
                content_type.parse().unwrap_or(header::HeaderValue::from_static("application/octet-stream"))
            );
            // Cache por 1 hora
            headers.insert(
                header::CACHE_CONTROL,
                "public, max-age=3600".parse().unwrap()
            );
            
            (StatusCode::OK, headers, data).into_response()
        }
        Err(e) => {
            warn!("⚠️ Archivo no encontrado: {} - {}", file_path, e);
            (StatusCode::NOT_FOUND, "Archivo no encontrado").into_response()
        }
    }
}

/// Actualiza el campo media de una agencia
async fn update_agencia_media(
    state: &AppState,
    agencia_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    use crate::application::dtos::UpdateAgenciaRequest;
    use serde_json::json;
    
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
    
    state.container.update_agencia_use_case
        .execute(agencia_id, request, updated_by)
        .await?;
    
    Ok(())
}
