use chrono::{DateTime, Utc};
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::{cancelaciones, saldos_favor, movimientos_saldo_favor};

// ==================== CANCELACION ====================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = cancelaciones)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CancelacionModel {
    pub id: i32,
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_total_file: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub monto_penalidad: BigDecimal,
    pub monto_saldo_favor: BigDecimal,
    pub tipo_cancelacion: String,
    pub hora_limite_cancelacion: Option<DateTime<Utc>>,
    pub motivo: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = cancelaciones)]
pub struct NewCancelacionModel {
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_total_file: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub monto_penalidad: BigDecimal,
    pub monto_saldo_favor: BigDecimal,
    pub tipo_cancelacion: String,
    pub hora_limite_cancelacion: Option<DateTime<Utc>>,
    pub motivo: Option<String>,
    pub notas: Option<String>,
    pub created_by: Option<i32>,
}

// ==================== SALDO FAVOR ====================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = saldos_favor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SaldoFavorModel {
    pub id: i32,
    pub id_agencia: i32,
    pub saldo_disponible: BigDecimal,
    pub saldo_utilizado: BigDecimal,
    pub saldo_total_generado: BigDecimal,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ==================== MOVIMIENTO SALDO FAVOR ====================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = movimientos_saldo_favor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MovimientoSaldoFavorModel {
    pub id: i32,
    pub id_saldo_favor: i32,
    pub id_agencia: i32,
    pub tipo: String,
    pub monto: BigDecimal,
    pub id_cancelacion: Option<i32>,
    pub id_file_destino: Option<i32>,
    pub id_pago_file: Option<i32>,
    pub saldo_anterior: BigDecimal,
    pub saldo_posterior: BigDecimal,
    pub concepto: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = movimientos_saldo_favor)]
pub struct NewMovimientoSaldoFavorModel {
    pub id_saldo_favor: i32,
    pub id_agencia: i32,
    pub tipo: String,
    pub monto: BigDecimal,
    pub id_cancelacion: Option<i32>,
    pub id_file_destino: Option<i32>,
    pub id_pago_file: Option<i32>,
    pub saldo_anterior: BigDecimal,
    pub saldo_posterior: BigDecimal,
    pub concepto: Option<String>,
    pub created_by: Option<i32>,
}
