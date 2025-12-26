//! # Persona Types - TypeScript Exports
//!
//! Tipos de persona exportables a TypeScript.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Información de persona
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PersonaTs {
    pub id: Uuid,
    pub nombres: String,
    pub apellido_paterno: String,
    pub apellido_materno: Option<String>,
    pub tipo_documento: Option<String>,
    pub numero_documento: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub sexo: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub nacionalidad: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear persona
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreatePersonaRequestTs {
    pub nombres: String,
    pub apellido_paterno: String,
    pub apellido_materno: Option<String>,
    pub tipo_documento: Option<String>,
    pub numero_documento: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub sexo: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub nacionalidad: Option<String>,
}

/// Request para actualizar persona
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdatePersonaRequestTs {
    pub nombres: Option<String>,
    pub apellido_paterno: Option<String>,
    pub apellido_materno: Option<String>,
    pub tipo_documento: Option<String>,
    pub numero_documento: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub sexo: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub nacionalidad: Option<String>,
}

/// Lista paginada de personas
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PersonaListResponseTs {
    pub personas: Vec<PersonaTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
