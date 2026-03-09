//! Modelos de contabilidad para Diesel
//!
//! Incluye:
//! - PagoFileModel: Pagos de agencias por files
//! - PagoProveedorModel: Pagos del admin a proveedores

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::{pagos_files, pagos_proveedores};

// ============================================================================
// PAGO FILE MODEL
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = pagos_files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PagoFileModel {
    pub id: i32,
    pub id_file: i32,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub estado: String,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub comprobante_url: Option<String>,
    pub comprobante_key: Option<String>,
    pub verificado_por: Option<i32>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub id_file_tour: Option<i32>,
    pub tipo_registro: String,
    pub monto_saldo_favor: Option<BigDecimal>,
    pub saldo_autorizado: bool,
    pub saldo_autorizado_por: Option<i32>,
    pub saldo_autorizado_at: Option<DateTime<Utc>>,
    pub entradas: bool,
    pub entrada_precio: Option<BigDecimal>,
    pub cuota: Option<i16>,
    pub id_entidad: i32,
    pub entidad: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = pagos_files)]
pub struct NewPagoFileModel<'a> {
    pub id_file: i32,
    pub id_entidad: i32,
    pub entidad: Option<&'a str>,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub estado: &'a str,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
    // Nuevos campos
    pub id_file_tour: Option<i32>,
    pub tipo_registro: &'a str,
    pub monto_saldo_favor: Option<BigDecimal>,
    pub saldo_autorizado: bool,
    pub saldo_autorizado_por: Option<i32>,
    pub saldo_autorizado_at: Option<DateTime<Utc>>,
    pub entradas: bool,
    pub entrada_precio: Option<BigDecimal>,
    pub cuota: Option<i16>,
}

#[derive(Debug, AsChangeset, Default)]
#[diesel(table_name = pagos_files)]
pub struct UpdatePagoFileModel<'a> {
    pub monto_total: Option<BigDecimal>,
    pub monto_pagado: Option<BigDecimal>,
    pub estado: Option<&'a str>,
    pub comprobante_url: Option<&'a str>,
    pub comprobante_key: Option<&'a str>,
    pub verificado_por: Option<i32>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<&'a str>,
    // Nuevos campos
    pub monto_saldo_favor: Option<BigDecimal>,
    pub saldo_autorizado: Option<bool>,
    pub saldo_autorizado_por: Option<i32>,
    pub saldo_autorizado_at: Option<DateTime<Utc>>,
    pub entradas: Option<bool>,
    pub entrada_precio: Option<Option<BigDecimal>>,
    pub tipo_registro: Option<&'a str>,
    pub cuota: Option<Option<i16>>,
}

// ============================================================================
// PAGO PROVEEDOR MODEL
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = pagos_proveedores)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PagoProveedorModel {
    pub id: i32,
    pub tipo_proveedor: String,
    pub id_transporte: Option<i32>,
    pub id_restaurante: Option<i32>,
    pub id_guia: Option<i32>,
    pub id_file_tour: Option<i32>,
    pub id_file_vehiculo: Option<i32>,
    pub id_file_restaurante: Option<i32>,
    pub id_file_guia: Option<i32>,
    pub monto_total: BigDecimal,
    pub estado: String,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
    pub comprobante_key: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub pagado_by: Option<i32>,
    pub id_entrada: Option<i32>,
    pub id_file_entrada: Option<i32>,
    pub monto_pagado: Option<BigDecimal>,
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
    pub monto_total: BigDecimal,
    pub estado: &'a str,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
    pub id_entrada: Option<i32>,
    pub id_file_entrada: Option<i32>,
}

#[derive(Debug, AsChangeset, Default)]
#[diesel(table_name = pagos_proveedores)]
pub struct UpdatePagoProveedorModel<'a> {
    pub monto_total: Option<BigDecimal>,
    pub estado: Option<&'a str>,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<&'a str>,
    pub comprobante_key: Option<&'a str>,
    pub notas: Option<&'a str>,
    pub pagado_by: Option<i32>,
    pub monto_pagado: Option<BigDecimal>,
}
