//! Ports (traits) para los repositorios de contabilidad
//!
//! Define interfaces para:
//! - CuentaRepositoryPort: Gestión de cuentas financieras
//! - MovimientoRepositoryPort: Registro de ingresos/egresos
//! - PagoFileRepositoryPort: Pagos de agencias por files
//! - PagoProveedorRepositoryPort: Pagos del admin a proveedores
//! - TarifaServicioRepositoryPort: Tarifas de venta vs costo

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::{
    CuentaModel, MovimientoModel, PagoFileModel, PagoProveedorModel, TarifaServicioModel,
    NewCuentaModel, NewMovimientoModel, NewPagoFileModel, NewPagoProveedorModel, NewTarifaServicioModel,
    UpdateCuentaModel, UpdatePagoFileModel, UpdatePagoProveedorModel, UpdateTarifaServicioModel,
};

// ============================================================================
// CUENTA REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait CuentaRepositoryPort: Send + Sync {
    /// Obtener cuenta por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<CuentaModel>, ApplicationError>;
    
    /// Obtener cuenta del admin (tipo = 'admin')
    async fn find_admin_account(&self) -> Result<Option<CuentaModel>, ApplicationError>;
    
    /// Obtener cuenta de una agencia
    async fn find_by_agencia(&self, id_agencia: i32) -> Result<Option<CuentaModel>, ApplicationError>;
    
    /// Listar todas las cuentas activas
    async fn find_all(&self) -> Result<Vec<CuentaModel>, ApplicationError>;
    
    /// Crear cuenta
    async fn create(&self, data: NewCuentaModel<'_>) -> Result<CuentaModel, ApplicationError>;
    
    /// Actualizar saldo de cuenta
    async fn update_saldo(&self, id: i32, nuevo_saldo: BigDecimal) -> Result<CuentaModel, ApplicationError>;
    
    /// Actualizar cuenta
    async fn update(&self, id: i32, data: UpdateCuentaModel<'_>) -> Result<CuentaModel, ApplicationError>;
}

// ============================================================================
// MOVIMIENTO REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait MovimientoRepositoryPort: Send + Sync {
    /// Obtener movimiento por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<MovimientoModel>, ApplicationError>;
    
    /// Listar movimientos de una cuenta
    async fn find_by_cuenta(
        &self, 
        id_cuenta: i32, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<MovimientoModel>, ApplicationError>;
    
    /// Listar movimientos con filtros
    async fn find_filtered(
        &self,
        id_cuenta: Option<i32>,
        tipo: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        referencia_tipo: Option<&str>,
        referencia_id: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MovimientoModel>, ApplicationError>;
    
    /// Contar movimientos con filtros
    async fn count_filtered(
        &self,
        id_cuenta: Option<i32>,
        tipo: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        referencia_tipo: Option<&str>,
        referencia_id: Option<i32>,
    ) -> Result<i64, ApplicationError>;
    
    /// Crear movimiento
    async fn create(&self, data: NewMovimientoModel<'_>) -> Result<MovimientoModel, ApplicationError>;
    
    /// Sumar ingresos en un período
    async fn sum_ingresos(
        &self, 
        id_cuenta: i32, 
        fecha_desde: DateTime<Utc>, 
        fecha_hasta: DateTime<Utc>
    ) -> Result<BigDecimal, ApplicationError>;
    
    /// Sumar egresos en un período
    async fn sum_egresos(
        &self, 
        id_cuenta: i32, 
        fecha_desde: DateTime<Utc>, 
        fecha_hasta: DateTime<Utc>
    ) -> Result<BigDecimal, ApplicationError>;

    /// Actualizar comprobante de movimiento
    async fn update_comprobante(
        &self,
        id: i32,
        comprobante_url: &str,
        comprobante_key: &str,
    ) -> Result<(), ApplicationError>;
}

// ============================================================================
// PAGO FILE REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait PagoFileRepositoryPort: Send + Sync {
    /// Obtener pago por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoFileModel>, ApplicationError>;
    
    /// Obtener pago de un file específico
    async fn find_by_file(&self, id_file: i32) -> Result<Option<PagoFileModel>, ApplicationError>;
    
    /// Listar pagos de una agencia
    async fn find_by_agencia(
        &self, 
        id_agencia: i32, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<PagoFileModel>, ApplicationError>;
    
    /// Listar pagos con filtros
    async fn find_filtered(
        &self,
        id_agencia: Option<i32>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoFileModel>, ApplicationError>;
    
    /// Contar pagos con filtros
    async fn count_filtered(
        &self,
        id_agencia: Option<i32>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
    ) -> Result<i64, ApplicationError>;
    
    /// Crear pago de file
    async fn create(&self, data: NewPagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError>;
    
    /// Actualizar pago
    async fn update(&self, id: i32, data: UpdatePagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError>;
    
    /// Sumar monto pendiente por cobrar (todos los files en estado pendiente/parcial)
    async fn sum_pendiente_cobrar(&self) -> Result<BigDecimal, ApplicationError>;
    
    /// Sumar monto pendiente de una agencia
    async fn sum_pendiente_agencia(&self, id_agencia: i32) -> Result<BigDecimal, ApplicationError>;
    
    /// Contar files pendientes de pago
    async fn count_pendientes(&self) -> Result<i64, ApplicationError>;
}

// ============================================================================
// PAGO PROVEEDOR REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait PagoProveedorRepositoryPort: Send + Sync {
    /// Obtener pago por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoProveedorModel>, ApplicationError>;
    
    /// Listar pagos de un proveedor (transporte, restaurante, guía)
    async fn find_by_proveedor(
        &self,
        tipo_proveedor: &str,
        id_proveedor: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoProveedorModel>, ApplicationError>;
    
    /// Listar pagos con filtros
    async fn find_filtered(
        &self,
        tipo_proveedor: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoProveedorModel>, ApplicationError>;
    
    /// Contar pagos con filtros
    async fn count_filtered(
        &self,
        tipo_proveedor: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
    ) -> Result<i64, ApplicationError>;
    
    /// Crear pago a proveedor
    async fn create(&self, data: NewPagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError>;
    
    /// Actualizar pago
    async fn update(&self, id: i32, data: UpdatePagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError>;
    
    /// Sumar monto pendiente por pagar
    async fn sum_pendiente_pagar(&self) -> Result<BigDecimal, ApplicationError>;
    
    /// Contar pagos pendientes
    async fn count_pendientes(&self) -> Result<i64, ApplicationError>;
    
    /// Buscar pago por file_tour y tipo
    async fn find_by_file_relation(
        &self,
        tipo_proveedor: &str,
        id_file_vehiculo: Option<i32>,
        id_file_restaurante: Option<i32>,
        id_file_guia: Option<i32>,
    ) -> Result<Option<PagoProveedorModel>, ApplicationError>;
}

// ============================================================================
// TARIFA SERVICIO REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait TarifaServicioRepositoryPort: Send + Sync {
    /// Obtener tarifa por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<TarifaServicioModel>, ApplicationError>;
    
    /// Obtener tarifa vigente de un servicio
    async fn find_vigente(
        &self,
        tipo_servicio: &str,
        id_servicio: i32,
        fecha: NaiveDate,
    ) -> Result<Option<TarifaServicioModel>, ApplicationError>;
    
    /// Listar tarifas de un tipo de servicio
    async fn find_by_tipo(
        &self,
        tipo_servicio: &str,
        solo_activas: bool,
    ) -> Result<Vec<TarifaServicioModel>, ApplicationError>;
    
    /// Listar todas las tarifas
    async fn find_all(&self, solo_activas: bool) -> Result<Vec<TarifaServicioModel>, ApplicationError>;
    
    /// Crear tarifa
    async fn create(&self, data: NewTarifaServicioModel<'_>) -> Result<TarifaServicioModel, ApplicationError>;
    
    /// Actualizar tarifa
    async fn update(&self, id: i32, data: UpdateTarifaServicioModel) -> Result<TarifaServicioModel, ApplicationError>;
    
    /// Desactivar tarifa
    async fn deactivate(&self, id: i32) -> Result<bool, ApplicationError>;
}
