//! # File Types - TypeScript Exports
//!
//! Tipos de archivos (documentos, imágenes) exportables a TypeScript.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Status del archivo
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum StatusFileTs {
    Pendiente,
    Procesando,
    Completado,
    Error,
}

/// Información de archivo
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct FileTs {
    pub id: Uuid,
    pub file_code: String,
    pub nombre_original: String,
    pub nombre_almacenado: Option<String>,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub size_bytes: Option<i64>,
    pub path: Option<String>,
    pub url: Option<String>,
    // Relación polimórfica
    pub entidad_tipo: Option<String>,
    pub entidad_id: Option<Uuid>,
    pub status: StatusFileTs,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear/subir archivo
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateFileRequestTs {
    pub nombre_original: String,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub size_bytes: Option<i64>,
    // Relación polimórfica
    pub entidad_tipo: Option<String>,
    pub entidad_id: Option<Uuid>,
}

/// Request para actualizar archivo
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileRequestTs {
    pub nombre_original: Option<String>,
    pub status: Option<StatusFileTs>,
    pub is_active: Option<bool>,
}

/// Lista paginada de archivos
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct FileListResponseTs {
    pub files: Vec<FileTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Respuesta de upload exitoso
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct FileUploadResponseTs {
    pub file: FileTs,
    pub upload_url: String,
}

/// URL firmada para descarga
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct FileDownloadUrlTs {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}
