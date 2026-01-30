//! Tipos comunes para Storage handlers

use serde::Serialize;

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
