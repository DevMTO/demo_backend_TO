//! Proxy para servir archivos de Tigris

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::warn;

use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

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
            warn!("Archivo no encontrado: {} - {}", file_path, e);
            (StatusCode::NOT_FOUND, "Archivo no encontrado").into_response()
        }
    }
}
