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

/// Respuesta de eliminación de archivo
#[derive(Debug, Serialize)]
pub struct StorageDeleteResponse {
    pub success: bool,
    pub message: String,
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
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth_user.user.role == UserRole::Agencias;
    
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
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth_user.user.role == UserRole::Agencias;
    
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

/// Eliminar logo de agencia
/// 
/// DELETE /api/v1/storage/agencia/{agencia_id}/logo
pub async fn delete_agencia_logo(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(agencia_id): Path<i32>,
) -> Result<Json<StorageDeleteResponse>, (StatusCode, Json<StorageErrorResponse>)> {
    info!("🗑️ Eliminando logo para agencia {}", agencia_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar permisos
    let is_agencia_user = auth_user.user.role == UserRole::Agencias;
    
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
            warn!("⚠️ Error eliminando archivo de Tigris: {}", e);
            // Continuamos para limpiar la BD aunque falle Tigris
        }
    }
    
    // Actualizar media quitando el logo
    if let Err(e) = clear_agencia_media(&state, agencia_id, "logo", auth_user.user.id).await {
        warn!("⚠️ Error limpiando media de agencia: {}", e);
    }
    
    info!("✅ Logo eliminado para agencia {}", agencia_id);
    
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
    info!("🗑️ Eliminando banner para agencia {}", agencia_id);
    
    let storage = state.container.tigris_storage.as_ref()
        .ok_or_else(|| {
            (StatusCode::SERVICE_UNAVAILABLE, Json(StorageErrorResponse {
                success: false,
                error: "Storage no disponible".to_string(),
            }))
        })?;
    
    // Verificar permisos
    let is_agencia_user = auth_user.user.role == UserRole::Agencias;
    
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
            warn!("⚠️ Error eliminando archivo de Tigris: {}", e);
        }
    }
    
    // Actualizar media quitando el banner
    if let Err(e) = clear_agencia_media(&state, agencia_id, "banner", auth_user.user.id).await {
        warn!("⚠️ Error limpiando media de agencia: {}", e);
    }
    
    info!("✅ Banner eliminado para agencia {}", agencia_id);
    
    Ok(Json(StorageDeleteResponse {
        success: true,
        message: "Banner eliminado correctamente".to_string(),
    }))
}

/// Limpia un campo de media de una agencia (lo pone en null)
async fn clear_agencia_media(
    state: &AppState,
    agencia_id: i32,
    field: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    use crate::application::dtos::UpdateAgenciaRequest;
    use serde_json::json;
    
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
    
    state.container.agencia_service
        .update_agencia(agencia_id, request, updated_by, None)
        .await?;
    
    Ok(())
}

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
            error!("❌ Storage no configurado");
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
                    info!("✅ Usuario {} es encargado de transporte {} (persona_id: {})", 
                        auth_user.user.username, transporte_id, user_persona_id);
                }
            }
        }
    }
    
    if !can_upload {
        warn!("⚠️ Usuario {} no tiene permisos para subir logo de transporte {}", 
            auth_user.user.username, transporte_id);
        return Err((StatusCode::FORBIDDEN, Json(StorageErrorResponse {
            success: false,
            error: "No tienes permisos para modificar este transporte".to_string(),
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
        let path = TigrisStorage::generate_transporte_path(transporte_id, "logo", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("❌ Error subiendo a Tigris: {}", e);
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
            warn!("⚠️ Error actualizando media de transporte: {}", e);
        }
        
        info!("✅ Logo de transporte subido: {}", url);
        
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

/// Actualiza el campo media de un transporte
async fn update_transporte_media(
    state: &AppState,
    transporte_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    use crate::application::dtos::UpdateTransporteRequest;
    use serde_json::json;
    
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
            error!("❌ Storage no configurado");
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
        warn!("⚠️ Usuario {} no tiene permisos para subir imagen de tour {}", 
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
            error!("❌ Error buscando tour: {}", e);
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
        let path = TigrisStorage::generate_tour_path(tour_id, "image", extension);
        
        let url = storage.upload(&path, &data, &content_type).await.map_err(|e| {
            error!("❌ Error subiendo a Tigris: {}", e);
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
            warn!("⚠️ Error actualizando media de tour: {}", e);
        }
        
        info!("✅ Imagen de tour subida: {}", url);
        
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

/// Actualiza el campo media de un tour
async fn update_tour_media(
    state: &AppState,
    tour_id: i32,
    field: &str,
    path: &str,
    updated_by: i32,
) -> Result<(), ApplicationError> {
    use crate::application::dtos::UpdateTourRequest;
    use serde_json::json;
    
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
        hora_inicio: None,
        hora_fin: None,
        detalles: None,
        itinerario: None,
        precio_base: None,
        duracion_dias: None,
        media: Some(media),
        tipo_tour: None,
        is_active: None,
    };
    
    // Aplicar la actualización usando el servicio
    state.container.tour_service
        .update_tour(tour_id, request, updated_by, None)
        .await?;
    
    Ok(())
}
