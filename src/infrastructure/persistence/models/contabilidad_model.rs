//! Modelos de contabilidad para Diesel
//! 
//! Incluye:
//! - CuentaModel: Cuentas financieras (admin y agencias)
//! - MovimientoModel: Registro de ingresos/egresos
//! - PagoFileModel: Pagos de agencias por files
//! - PagoProveedorModel: Pagos del admin a proveedores
//! - TarifaServicioModel: Precios de venta vs costo

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::{
    cuentas, movimientos, pagos_files, pagos_proveedores, tarifas_servicios,
};

// ============================================================================
// CUENTA MODEL
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = cuentas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CuentaModel {
    pub id: i32,
    pub nombre: String,
    pub tipo: String,  // 'admin', 'agencia'
    pub id_agencia: Option<i32>,
    pub saldo_actual: BigDecimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = cuentas)]
pub struct NewCuentaModel<'a> {
    pub nombre: &'a str,
    pub tipo: &'a str,
    pub id_agencia: Option<i32>,
    pub saldo_actual: BigDecimal,
    pub is_active: bool,
    pub created_by: Option<i32>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = cuentas)]
pub struct UpdateCuentaModel<'a> {
    pub nombre: Option<&'a str>,
    pub saldo_actual: Option<BigDecimal>,
    pub is_active: Option<bool>,
}

// ============================================================================
// MOVIMIENTO MODEL
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = movimientos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MovimientoModel {
    pub id: i32,
    pub id_cuenta: i32,
    pub tipo: String,  // 'ingreso', 'egreso'
    pub monto: BigDecimal,
    pub concepto: String,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<i32>,
    pub fecha_movimiento: DateTime<Utc>,
    pub saldo_anterior: BigDecimal,
    pub saldo_posterior: BigDecimal,
    pub notas: Option<String>,
    pub comprobante_url: Option<String>,
    pub comprobante_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = movimientos)]
pub struct NewMovimientoModel<'a> {
    pub id_cuenta: i32,
    pub tipo: &'a str,
    pub monto: BigDecimal,
    pub concepto: &'a str,
    pub referencia_tipo: Option<&'a str>,
    pub referencia_id: Option<i32>,
    pub fecha_movimiento: DateTime<Utc>,
    pub saldo_anterior: BigDecimal,
    pub saldo_posterior: BigDecimal,
    pub notas: Option<&'a str>,
    pub comprobante_url: Option<&'a str>,
    pub comprobante_key: Option<&'a str>,
    pub created_by: Option<i32>,
}

// ============================================================================
// PAGO FILE MODEL (Pagos de agencias por files)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = pagos_files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PagoFileModel {
    pub id: i32,
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub estado: String,  // 'pendiente', 'parcial', 'pagado', 'vencido'
    pub fecha_vencimiento: Option<NaiveDate>,
    pub comprobante_url: Option<String>,
    pub comprobante_key: Option<String>,
    pub verificado_por: Option<i32>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = pagos_files)]
pub struct NewPagoFileModel<'a> {
    pub id_file: i32,
    pub id_agencia: i32,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub estado: &'a str,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
}

#[derive(Debug, AsChangeset, Default)]
#[diesel(table_name = pagos_files)]
pub struct UpdatePagoFileModel<'a> {
    pub monto_pagado: Option<BigDecimal>,
    pub estado: Option<&'a str>,
    pub comprobante_url: Option<&'a str>,
    pub comprobante_key: Option<&'a str>,
    pub verificado_por: Option<i32>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<&'a str>,
}

// ============================================================================
// PAGO PROVEEDOR MODEL (Pagos del admin a transportes/restaurantes/guías)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = pagos_proveedores)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PagoProveedorModel {
    pub id: i32,
    pub tipo_proveedor: String,  // 'transporte', 'restaurante', 'guia'
    pub id_transporte: Option<i32>,
    pub id_restaurante: Option<i32>,
    pub id_guia: Option<i32>,
    pub id_file_tour: Option<i32>,
    pub id_file_vehiculo: Option<i32>,
    pub id_file_restaurante: Option<i32>,
    pub id_file_guia: Option<i32>,
    pub monto: BigDecimal,
    pub estado: String,  // 'pendiente', 'pagado'
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
    pub comprobante_key: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub pagado_by: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = pagos_proveedores)]
pub struct NewPagoProveedorModel<'a> {
    pub tipo_proveedor: &'a str,
    pub id_transporte: Option<i32>,
    pub id_restaurante: Option<i32>,
    pub id_guia: Option<i32>,
    pub id_file_tour: Option<i32>,
    pub id_file_vehiculo: Option<i32>,
    pub id_file_restaurante: Option<i32>,
    pub id_file_guia: Option<i32>,
    pub monto: BigDecimal,
    pub estado: &'a str,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
}

#[derive(Debug, AsChangeset, Default)]
#[diesel(table_name = pagos_proveedores)]
pub struct UpdatePagoProveedorModel<'a> {
    pub estado: Option<&'a str>,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<&'a str>,
    pub comprobante_key: Option<&'a str>,
    pub notas: Option<&'a str>,
    pub pagado_by: Option<i32>,
}

// ============================================================================
// TARIFA SERVICIO MODEL (Precios de venta vs costo)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tarifas_servicios)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TarifaServicioModel {
    pub id: i32,
    pub tipo_servicio: String,  // 'tour', 'entrada', 'restaurante', 'transporte', 'guia'
    pub id_servicio: i32,
    pub precio_venta: BigDecimal,
    pub precio_costo: BigDecimal,
    pub margen: BigDecimal,  // GENERATED ALWAYS AS (precio_venta - precio_costo)
    pub vigente_desde: NaiveDate,
    pub vigente_hasta: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tarifas_servicios)]
pub struct NewTarifaServicioModel<'a> {
    pub tipo_servicio: &'a str,
    pub id_servicio: i32,
    pub precio_venta: BigDecimal,
    pub precio_costo: BigDecimal,
    pub vigente_desde: NaiveDate,
    pub vigente_hasta: Option<NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<i32>,
}

#[derive(Debug, AsChangeset, Default)]
#[diesel(table_name = tarifas_servicios)]
pub struct UpdateTarifaServicioModel {
    pub precio_venta: Option<BigDecimal>,
    pub precio_costo: Option<BigDecimal>,
    pub vigente_hasta: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

// ============================================================================
// MODELOS CON RELACIONES (para consultas complejas)
// ============================================================================

/// Movimiento con información de cuenta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovimientoWithCuentaModel {
    pub movimiento: MovimientoModel,
    pub cuenta_nombre: String,
    pub cuenta_tipo: String,
}

/// Pago de file con información de agencia y file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagoFileWithDetailsModel {
    pub pago: PagoFileModel,
    pub file_code: Option<String>,
    pub agencia_nombre: String,
    pub verificador_username: Option<String>,
}

/// Pago a proveedor con detalles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagoProveedorWithDetailsModel {
    pub pago: PagoProveedorModel,
    pub proveedor_nombre: String,
    pub file_code: Option<String>,
    pub tour_nombre: Option<String>,
}
