//! # Restaurante Types - TypeScript Exports
//!
//! Tipos de restaurante exportables a TypeScript.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Información de restaurante
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct RestauranteTs {
    pub id: Uuid,
    pub codigo: String,
    pub nombre: String,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub tipo_cocina: Option<String>,
    pub capacidad: Option<i32>,
    pub precio_promedio: Option<f64>,
    pub horario_apertura: Option<String>,
    pub horario_cierre: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear restaurante
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateRestauranteRequestTs {
    pub nombre: String,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub tipo_cocina: Option<String>,
    pub capacidad: Option<i32>,
    pub precio_promedio: Option<f64>,
    pub horario_apertura: Option<String>,
    pub horario_cierre: Option<String>,
}

/// Request para actualizar restaurante
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateRestauranteRequestTs {
    pub nombre: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub tipo_cocina: Option<String>,
    pub capacidad: Option<i32>,
    pub precio_promedio: Option<f64>,
    pub horario_apertura: Option<String>,
    pub horario_cierre: Option<String>,
    pub is_active: Option<bool>,
}

/// Lista paginada de restaurantes
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct RestauranteListResponseTs {
    pub restaurantes: Vec<RestauranteTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
