//! # Agencia Types - TypeScript Exports
//!
//! Tipos de agencia exportables a TypeScript.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Información de agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct AgenciaTs {
    pub id: Uuid,
    pub razon_social: String,
    pub nombre_comercial: String,
    pub ruc: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub web: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateAgenciaRequestTs {
    pub razon_social: String,
    pub nombre_comercial: String,
    pub ruc: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub web: Option<String>,
}

/// Request para actualizar agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgenciaRequestTs {
    pub razon_social: Option<String>,
    pub nombre_comercial: Option<String>,
    pub ruc: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub web: Option<String>,
    pub is_active: Option<bool>,
}

/// Lista paginada de agencias
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct AgenciaListResponseTs {
    pub agencias: Vec<AgenciaTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
