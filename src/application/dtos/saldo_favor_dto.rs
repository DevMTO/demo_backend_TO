//! DTOs para Saldo a Favor, Cancelaciones y No Shows

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ==================== CANCELACION DTOs ====================

/// Respuesta de cancelación
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CancelacionResponse {
    pub id: i32,
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_total_file: f64,
    pub monto_pagado: f64,
    pub monto_saldo_favor: f64,
    pub monto_operador: f64,
    pub tipo_cancelacion: String,
    pub motivo: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Info del file (populated via JOIN)
    pub file_code: Option<String>,
    pub agencia_nombre: Option<String>,
}

/// Request para cancelar un file (cancelación normal, antes de 8PM)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CancelarFileRequest {
    pub id_file: i32,
    pub motivo: Option<String>,
    pub notas: Option<String>,
}

/// Request para registrar un no_show (admin, después de 8PM)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RegistrarNoShowRequest {
    pub id_file: i32,
    pub motivo: Option<String>,
    pub notas: Option<String>,
}

// ==================== NO SHOW DTOs ====================

/// Respuesta de un no_show con desglose
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct NoShowResponse {
    pub id: i32,
    pub id_cancelacion: i32,
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_restaurantes: f64,
    pub monto_entradas: f64,
    pub monto_saldo_favor: f64,
    pub monto_operador: f64,
    pub fecha_inicio_file: NaiveDate,
    pub hora_corte: DateTime<Utc>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Info adicional
    pub file_code: Option<String>,
    pub agencia_nombre: Option<String>,
}

// ==================== SALDO FAVOR DTOs ====================

/// Respuesta del saldo a favor de una agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct SaldoFavorResponse {
    pub id: i32,
    pub id_agencia: i32,
    pub agencia_nombre: Option<String>,
    pub saldo_disponible: f64,
    pub saldo_utilizado: f64,
    pub saldo_total_generado: f64,
    pub updated_at: DateTime<Utc>,
}

/// Respuesta de un movimiento de saldo a favor
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MovimientoSaldoFavorResponse {
    pub id: i32,
    pub id_agencia: i32,
    pub tipo: String,
    pub monto: f64,
    pub id_cancelacion: Option<i32>,
    pub id_file_destino: Option<i32>,
    pub id_pago_file: Option<i32>,
    pub saldo_anterior: f64,
    pub saldo_posterior: f64,
    pub concepto: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Info adicional (via JOIN)
    pub file_code_origen: Option<String>,
    pub file_code_destino: Option<String>,
}

/// Request para usar saldo a favor en un pago
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UsarSaldoFavorRequest {
    pub id_agencia: i32,
    pub id_file_destino: i32,
    pub id_pago_file: Option<i32>,
    pub monto: f64,
    pub concepto: Option<String>,
}

/// Dashboard de saldo a favor para una agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct SaldoFavorDashboard {
    pub saldo: SaldoFavorResponse,
    pub cancelaciones_recientes: Vec<CancelacionResponse>,
    pub movimientos_recientes: Vec<MovimientoSaldoFavorResponse>,
    pub total_cancelaciones: i64,
    pub total_no_shows: i64,
}

/// Filtro para listar cancelaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelacionesFilter {
    pub id_agencia: Option<i32>,
    pub tipo_cancelacion: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Filtro para listar movimientos de saldo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovimientosSaldoFilter {
    pub id_agencia: Option<i32>,
    pub tipo: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
