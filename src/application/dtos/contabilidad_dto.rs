//! DTOs para el módulo de contabilidad
//!
//! Incluye requests y responses para:
//! - Dashboard contabilidad agencia
//! - Pagos de files (agencias al admin)
//! - Pagos a proveedores (admin a transportes/restaurantes/guias)

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ============================================================================
// DASHBOARD AGENCIA
// ============================================================================

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
    /// Si la agencia tiene pago anticipado
    pub pago_anticipado: bool,
    /// Tipo de vencimiento: semanal, quincenal, mensual (cuando no es anticipado)
    pub tipo_vencimiento: Option<String>,
    /// Files pendientes de pago
    pub files_pendientes: Vec<PagoFileResponse>,
    /// Ultimos pagos realizados
    pub ultimos_pagos: Vec<PagoFileResponse>,
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
    pub estado: String,
    pub fecha_vencimiento: Option<String>,
    pub comprobante_url: Option<String>,
    pub verificado_por: Option<i32>,
    pub verificador_nombre: Option<String>,
    pub verificado_at: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    /// ID del file_tour asociado (para deudas por tour)
    pub id_file_tour: Option<i32>,
    /// Nombre del tour (para deudas por tour)
    pub tour_nombre: Option<String>,
    /// Tipo de registro: deuda, pago, cancelacion, etc.
    pub tipo_registro: String,
    /// Si este registro cubre entradas
    pub entradas: bool,
    /// Costo de las entradas del file_tour (solo si entradas = true)
    #[ts(type = "number | null")]
    pub entrada_precio: Option<f64>,
    /// Número de cuota (para indexar pagos de un file_tour)
    pub cuota: Option<i16>,
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
    /// Comprobante en base64 (se subira a Tigris)
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
// PAGOS A PROVEEDORES (Admin -> Transportes/Restaurantes/Guias)
// ============================================================================

/// Response de pago a proveedor
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagoProveedorResponse {
    pub id: i32,
    pub tipo_proveedor: String,
    pub proveedor_id: i32,
    pub proveedor_nombre: Option<String>,
    pub id_file_tour: Option<i32>,
    pub id_file_vehiculo: Option<i32>,
    pub id_file_restaurante: Option<i32>,
    pub id_file_guia: Option<i32>,
    pub file_code: Option<String>,
    pub tour_nombre: Option<String>,
    pub fecha_tour: Option<String>,
    #[ts(type = "string")]
    pub monto: BigDecimal,
    pub estado: String,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub comprobante_url: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub pagado_por: Option<String>,
}

/// Request para crear pago a proveedor
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePagoProveedorRequest {
    pub tipo_proveedor: String,
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
    /// Comprobante en base64 (se subira a Tigris)
    pub comprobante_base64: Option<String>,
    pub comprobante_filename: Option<String>,
}

/// Request para marcar pago a proveedor como pagado
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MarcarPagoProveedorPagadoRequest {
    /// Monto real a pagar (si difiere del monto original del servicio)
    #[ts(type = "string | undefined")]
    pub monto: Option<BigDecimal>,
    /// Notas adicionales sobre el pago
    pub notas: Option<String>,
    /// URL del comprobante de pago (si ya se subio)
    pub comprobante_url: Option<String>,
}

// ============================================================================
// MIS PAGOS (Vista de proveedores)
// ============================================================================

/// Vista de pago para un guia
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
    pub estado: String,
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

/// Filtros para listar pagos de files
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PagosFilesFilter {
    pub id_agencia: Option<i32>,
    pub estado: Option<String>,
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

// ============================================================================
// SALDO A FAVOR - CANCELACIONES Y NO-SHOWS
// ============================================================================

/// Respuesta de cancelación (registro en pagos_files con tipo_registro='cancelacion'/'cancelacion_tour')
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CancelacionResponse {
    pub id: i32,
    pub id_file: i32,
    pub file_code: Option<String>,
    pub id_agencia: i32,
    pub agencia_nombre: Option<String>,
    pub id_file_tour: Option<i32>,
    pub tour_nombre: Option<String>,
    #[ts(type = "number")]
    pub monto_total: f64,
    #[ts(type = "number")]
    pub monto_saldo_favor: f64,
    /// Monto total de entradas asociadas (calculado desde file_entradas × entrada_precios)
    #[ts(type = "number")]
    pub monto_entradas: f64,
    pub tipo_cancelacion: String,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Monto de entradas BTG/BTP transferidas al siguiente tour
    #[ts(type = "number")]
    pub monto_entradas_transferidas: f64,
    /// ID del file_tour al que se transfirieron las entradas BTG/BTP
    pub id_file_tour_destino: Option<i32>,
}

/// Request para cancelar un file completo
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CancelarFileRequest {
    pub id_file: i32,
    pub notas: Option<String>,
}

/// Request para cancelar un file_tour específico
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CancelarFileTourRequest {
    pub id_file_tour: i32,
    pub notas: Option<String>,
}

/// Respuesta de No-Show (registro en pagos_files con tipo_registro='no_show'/'no_show_tour')
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct NoShowResponse {
    pub id: i32,
    pub id_file: i32,
    pub file_code: Option<String>,
    pub id_agencia: i32,
    pub agencia_nombre: Option<String>,
    pub id_file_tour: Option<i32>,
    pub tour_nombre: Option<String>,
    #[ts(type = "number")]
    pub monto_total: f64,
    #[ts(type = "number")]
    pub monto_saldo_favor: f64,
    /// Monto total de entradas asociadas (calculado desde file_entradas × entrada_precios)
    #[ts(type = "number")]
    pub monto_entradas: f64,
    pub saldo_autorizado: bool,
    pub saldo_autorizado_por: Option<i32>,
    pub saldo_autorizado_at: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request para registrar no-show de un file completo
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RegistrarNoShowRequest {
    pub id_file: i32,
    pub notas: Option<String>,
}

/// Request para registrar no-show de un file_tour específico
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct NoShowFileTourRequest {
    pub id_file_tour: i32,
    pub notas: Option<String>,
}

/// Request para autorizar saldo a favor de un no-show
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AutorizarNoShowSaldoRequest {
    pub id_pago_file: i32,
    #[ts(type = "number")]
    pub monto_saldo_favor: f64,
}

/// Resumen de saldo a favor de una agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct SaldoFavorResumen {
    pub id_agencia: i32,
    pub nombre_agencia: String,
    #[ts(type = "number")]
    pub saldo_generado: f64,
    #[ts(type = "number")]
    pub saldo_usado: f64,
    #[ts(type = "number")]
    pub saldo_disponible: f64,
    pub total_cancelaciones: i32,
    pub total_no_shows: i32,
}

/// Respuesta de movimiento de saldo a favor
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MovimientoSaldoResponse {
    pub id: i32,
    pub id_file: i32,
    pub file_code: Option<String>,
    pub id_agencia: i32,
    pub id_file_tour: Option<i32>,
    pub tipo: String,
    pub concepto: String,
    #[ts(type = "number")]
    pub monto: f64,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Dashboard de saldo a favor para una agencia
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct SaldoFavorDashboard {
    pub resumen: SaldoFavorResumen,
    pub cancelaciones_recientes: Vec<CancelacionResponse>,
    pub no_shows_recientes: Vec<NoShowResponse>,
    pub movimientos_recientes: Vec<MovimientoSaldoResponse>,
}

/// Request para usar saldo a favor en un pago
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UsarSaldoFavorRequest {
    pub id_agencia: i32,
    pub id_file: i32,
    #[ts(type = "number")]
    pub monto: f64,
    pub concepto: Option<String>,
}
