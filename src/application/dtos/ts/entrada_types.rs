use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntradaRequestTs {
    pub monto: Option<f64>,
    pub notas: Option<String>,
    pub is_active: Option<bool>,
}

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
