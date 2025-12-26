//! # Entrada Types - TypeScript Exports
//!
//! Tipos de entrada (pasajero/cliente en tour) exportables a TypeScript.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Información de entrada (registro de pasajero en un tour)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct EntradaTs {
    pub id: Uuid,
    pub codigo: String,
    pub id_tour: Uuid,
    pub id_persona: Uuid,
    pub id_agencia: Option<Uuid>,
    pub monto: Option<f64>,
    pub monto_pagado: Option<f64>,
    pub notas: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear entrada
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateEntradaRequestTs {
    pub id_tour: Uuid,
    pub id_persona: Uuid,
    pub id_agencia: Option<Uuid>,
    pub monto: Option<f64>,
    pub notas: Option<String>,
}

/// Request para actualizar entrada
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntradaRequestTs {
    pub monto: Option<f64>,
    pub notas: Option<String>,
    pub is_active: Option<bool>,
}

/// Lista paginada de entradas
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct EntradaListResponseTs {
    pub entradas: Vec<EntradaTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Entrada con detalles expandidos
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct EntradaDetailTs {
    pub entrada: EntradaTs,
    pub persona_nombre_completo: String,
    pub persona_documento: Option<String>,
    pub persona_nacionalidad: Option<String>,
    pub tour_nombre: String,
    pub agencia_nombre: Option<String>,
    pub saldo_pendiente: f64,
}

/// Estadísticas de entradas por tour
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct EntradaStatsTs {
    pub total_entradas: i64,
    pub entradas_activas: i64,
    pub monto_total: f64,
    pub monto_pagado_total: f64,
    pub saldo_pendiente_total: f64,
}
