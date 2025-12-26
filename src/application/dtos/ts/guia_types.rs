//! # Guía Types - TypeScript Exports
//!
//! Tipos de guía turístico exportables a TypeScript.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Status del guía
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum StatusGuiaTs {
    Disponible,
    EnServicio,
    Descanso,
    Inactivo,
}

/// Información de guía
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaTs {
    pub id: Uuid,
    pub codigo: String,
    pub id_persona: Uuid,
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub status: StatusGuiaTs,
    pub id_agencia: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear guía
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateGuiaRequestTs {
    pub id_persona: Uuid,
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub id_agencia: Option<Uuid>,
}

/// Request para actualizar guía
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuiaRequestTs {
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub status: Option<StatusGuiaTs>,
}

/// Lista paginada de guías
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaListResponseTs {
    pub guias: Vec<GuiaTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Guía con datos de persona expandidos
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaDetailTs {
    pub guia: GuiaTs,
    pub persona_nombre_completo: String,
    pub persona_documento: Option<String>,
    pub persona_telefono: Option<String>,
    pub persona_email: Option<String>,
}
