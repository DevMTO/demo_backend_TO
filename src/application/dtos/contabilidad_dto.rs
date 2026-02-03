//! DTOs para el módulo de contabilidad
//!
//! Incluye requests y responses para:
//! - Dashboard contabilidad admin
//! - Dashboard contabilidad agencia
//! - Pagos de files (agencias al admin)
//! - Pagos a proveedores (admin a transportes/restaurantes/guías)
//! - Movimientos financieros
//! - Tarifas de servicios

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ============================================================================
// DASHBOARD ADMIN
// ============================================================================

/// Dashboard de contabilidad para el admin/operador
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AdminContabilidadDashboard {
    /// Saldo actual de la cuenta del operador
    #[ts(type = "string")]
    pub saldo_actual: BigDecimal,
    /// Total de ingresos del período
    #[ts(type = "string")]
    pub total_ingresos: BigDecimal,
    /// Total de egresos del período
    #[ts(type = "string")]
    pub total_egresos: BigDecimal,
    /// Balance del período (ingresos - egresos)
    #[ts(type = "string")]
    pub balance_periodo: BigDecimal,
    /// Total pendiente por cobrar de agencias
    #[ts(type = "string")]
    pub total_pendiente_cobrar: BigDecimal,
    /// Total pendiente por pagar a proveedores
    #[ts(type = "string")]
    pub total_pendiente_pagar: BigDecimal,
    /// Cantidad de files pendientes de pago
    pub files_pendientes_pago: i32,
    /// Cantidad de pagos a proveedores pendientes
    pub pagos_proveedores_pendientes: i32,
    /// Últimos movimientos (top 10)
    pub ultimos_movimientos: Vec<MovimientoResponse>,
}

/// Dashboard de contabilidad para agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AgenciaContabilidadDashboard {
    /// ID de la agencia
    pub id_agencia: i32,
    /// Nombre de la agencia
    pub nombre_agencia: String,
    /// Total de files generados
    pub total_files: i32,
    /// Monto total de todos los files
    #[ts(type = "string")]
    pub monto_total_files: BigDecimal,
    /// Monto ya pagado
    #[ts(type = "string")]
    pub monto_pagado: BigDecimal,
    /// Monto pendiente por pagar
    #[ts(type = "string")]
    pub monto_pendiente: BigDecimal,
    /// Files pendientes de pago
    pub files_pendientes: Vec<PagoFileResponse>,
    /// Últimos pagos realizados
    pub ultimos_pagos: Vec<PagoFileResponse>,
}

// ============================================================================
// MOVIMIENTOS
// ============================================================================

/// Response de movimiento financiero
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MovimientoResponse {
    pub id: i32,
    pub id_cuenta: i32,
    pub cuenta_nombre: Option<String>,
    pub tipo: String,  // 'ingreso', 'egreso'
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub concepto: String,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<i32>,
    pub fecha_movimiento: DateTime<Utc>,
    #[ts(type = "string")]
    pub saldo_anterior: BigDecimal,
    #[ts(type = "string")]
    pub saldo_posterior: BigDecimal,
    pub notas: Option<String>,
    pub comprobante_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request para crear movimiento manual (ajustes)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateMovimientoRequest {
    pub id_cuenta: i32,
    pub tipo: String,  // 'ingreso', 'egreso'
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub concepto: String,
    pub notas: Option<String>,
    /// Comprobante en base64 (se subirá a Tigris)
    pub comprobante_base64: Option<String>,
    pub comprobante_filename: Option<String>,
}

// ============================================================================
// PAGOS DE FILES (Agencias -> Admin)
// ============================================================================

/// Response de pago de file
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagoFileResponse {
    pub id: i32,
    pub id_file: i32,
    pub file_code: Option<String>,
    pub id_agencia: i32,
    pub agencia_nombre: Option<String>,
    #[ts(type = "string")]
    pub monto_total: BigDecimal,
    #[ts(type = "string")]
    pub monto_pagado: BigDecimal,
    #[ts(type = "string")]
    pub monto_pendiente: BigDecimal,
    pub estado: String,  // 'pendiente', 'parcial', 'pagado', 'vencido'
    pub fecha_vencimiento: Option<String>,
    pub comprobante_url: Option<String>,
    pub verificado_por: Option<i32>,
    pub verificador_nombre: Option<String>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request para registrar pago de file (agencia sube comprobante)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RegistrarPagoFileRequest {
    pub id_pago_file: i32,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub notas: Option<String>,
    /// Comprobante en base64 (se subirá a Tigris)
    pub comprobante_base64: Option<String>,
    pub comprobante_filename: Option<String>,
}

/// Request para verificar pago de file (admin verifica)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct VerificarPagoFileRequest {
    pub id_pago_file: i32,
    pub aprobado: bool,
    pub notas: Option<String>,
}

// ============================================================================
// PAGOS A PROVEEDORES (Admin -> Transportes/Restaurantes/Guías)
// ============================================================================

/// Response de pago a proveedor
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagoProveedorResponse {
    pub id: i32,
    pub tipo_proveedor: String,  // 'transporte', 'restaurante', 'guia'
    pub proveedor_id: i32,
    pub proveedor_nombre: Option<String>,
    pub id_file_tour: Option<i32>,
    pub file_code: Option<String>,
    pub tour_nombre: Option<String>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub estado: String,  // 'pendiente', 'pagado'
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub pagado_por: Option<String>,
}

/// Request para crear pago a proveedor (al asignar servicio)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePagoProveedorRequest {
    pub tipo_proveedor: String,  // 'transporte', 'restaurante', 'guia'
    pub id_transporte: Option<i32>,
    pub id_restaurante: Option<i32>,
    pub id_guia: Option<i32>,
    pub id_file_tour: Option<i32>,
    pub id_file_vehiculo: Option<i32>,
    pub id_file_restaurante: Option<i32>,
    pub id_file_guia: Option<i32>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub notas: Option<String>,
}

/// Request para registrar pago a proveedor (admin paga)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagarProveedorRequest {
    pub id_pago_proveedor: i32,
    pub notas: Option<String>,
    /// Comprobante en base64 (se subirá a Tigris)
    pub comprobante_base64: Option<String>,
    pub comprobante_filename: Option<String>,
}

/// Request para marcar pago a proveedor como pagado
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MarcarPagoProveedorPagadoRequest {
    /// Notas adicionales sobre el pago
    pub notas: Option<String>,
    /// URL del comprobante de pago (si ya se subió)
    pub comprobante_url: Option<String>,
}

// ============================================================================
// TARIFAS DE SERVICIOS
// ============================================================================

/// Response de tarifa de servicio
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TarifaServicioResponse {
    pub id: i32,
    pub tipo_servicio: String,  // 'tour', 'entrada', 'restaurante', 'transporte', 'guia'
    pub id_servicio: i32,
    pub servicio_nombre: Option<String>,
    #[ts(type = "string")]
    pub precio_venta: BigDecimal,
    #[ts(type = "string")]
    pub precio_costo: BigDecimal,
    #[ts(type = "string | null")]
    pub margen: Option<BigDecimal>,
    pub vigente_desde: String,
    pub vigente_hasta: Option<String>,
    pub is_active: bool,
}

/// Request para crear/actualizar tarifa
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateTarifaServicioRequest {
    pub tipo_servicio: String,
    pub id_servicio: i32,
    #[ts(type = "string")]
    pub precio_venta: BigDecimal,
    #[ts(type = "string")]
    pub precio_costo: BigDecimal,
    pub vigente_desde: String,  // YYYY-MM-DD
    pub vigente_hasta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateTarifaServicioRequest {
    #[ts(type = "string | null")]
    pub precio_venta: Option<BigDecimal>,
    #[ts(type = "string | null")]
    pub precio_costo: Option<BigDecimal>,
    pub vigente_hasta: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// MIS PAGOS (Vista de proveedores)
// ============================================================================

/// Vista de pago para un guía
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MiPagoGuiaResponse {
    pub id_pago: i32,
    pub id_file_guia: i32,
    pub file_code: Option<String>,
    pub tour_nombre: String,
    pub fecha_tour: Option<String>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub estado: String,  // 'pendiente', 'pagado'
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
}

/// Vista de pago para un conductor/transporte
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MiPagoConductorResponse {
    pub id_pago: i32,
    pub id_file_vehiculo: i32,
    pub file_code: Option<String>,
    pub tour_nombre: String,
    pub vehiculo_placa: String,
    pub fecha_tour: Option<String>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub estado: String,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
}

/// Vista de pago para un restaurante
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MiPagoRestauranteResponse {
    pub id_pago: i32,
    pub id_file_restaurante: i32,
    pub file_code: Option<String>,
    pub tour_nombre: String,
    pub fecha_tour: Option<String>,
    pub tipo_servicio: Option<String>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub estado: String,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
}

// ============================================================================
// LISTADOS Y FILTROS
// ============================================================================

/// Filtros para listar movimientos
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MovimientosFilter {
    pub id_cuenta: Option<i32>,
    pub tipo: Option<String>,  // 'ingreso', 'egreso'
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    pub referencia_tipo: Option<String>,
}

/// Filtros para listar pagos de files
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagosFilesFilter {
    pub id_agencia: Option<i32>,
    pub estado: Option<String>,  // 'pendiente', 'parcial', 'pagado', 'vencido'
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
}

/// Filtros para listar pagos a proveedores
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagosProveedoresFilter {
    pub tipo_proveedor: Option<String>,
    pub estado: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
}

/// Lista paginada genérica para contabilidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedContabilidadResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

// ============================================================================
// RESUMEN FINANCIERO
// ============================================================================

/// Resumen financiero por período
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ResumenFinanciero {
    pub periodo: String,  // "2025-01", "2025-Q1", "2025"
    #[ts(type = "string")]
    pub total_ingresos: BigDecimal,
    #[ts(type = "string")]
    pub total_egresos: BigDecimal,
    #[ts(type = "string")]
    pub balance: BigDecimal,
    pub cantidad_files: i32,
    pub cantidad_pagos_proveedores: i32,
}
