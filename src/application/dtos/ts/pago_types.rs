use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum TipoMovimientoTs {
    Ingreso,
    Egreso,
    Ajuste,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoTs {
    pub id: Uuid,
    pub codigo: String,
    pub id_entrada: Uuid,
    pub tipo_movimiento: TipoMovimientoTs,
    pub monto: f64,
    pub fecha_pago: Option<NaiveDate>,
    pub metodo_pago: Option<String>,
    pub referencia: Option<String>,
    pub notas: Option<String>,
    pub id_usuario_registro: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreatePagoRequestTs {
    pub id_entrada: Uuid,
    pub tipo_movimiento: TipoMovimientoTs,
    pub monto: f64,
    pub fecha_pago: Option<NaiveDate>,
    pub metodo_pago: Option<String>,
    pub referencia: Option<String>,
    pub notas: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdatePagoRequestTs {
    pub tipo_movimiento: Option<TipoMovimientoTs>,
    pub monto: Option<f64>,
    pub fecha_pago: Option<NaiveDate>,
    pub metodo_pago: Option<String>,
    pub referencia: Option<String>,
    pub notas: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoListResponseTs {
    pub pagos: Vec<PagoTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoDetailTs {
    pub pago: PagoTs,
    pub entrada_codigo: String,
    pub persona_nombre_completo: String,
    pub tour_nombre: String,
    pub usuario_registro_nombre: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoResumenEntradaTs {
    pub id_entrada: Uuid,
    pub entrada_codigo: String,
    pub monto_total: f64,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    pub total_ajustes: f64,
    pub monto_pagado: f64,
    pub saldo_pendiente: f64,
    pub cantidad_pagos: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoResumenTourTs {
    pub id_tour: Uuid,
    pub tour_nombre: String,
    pub total_entradas: i64,
    pub monto_esperado: f64,
    pub monto_recaudado: f64,
    pub saldo_pendiente: f64,
    pub porcentaje_recaudacion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct PagoFiltersTs {
    pub id_entrada: Option<Uuid>,
    pub id_tour: Option<Uuid>,
    pub tipo_movimiento: Option<TipoMovimientoTs>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub metodo_pago: Option<String>,
}
