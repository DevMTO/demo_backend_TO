use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::{
    PagoFileModel, PagoProveedorModel,
    NewPagoFileModel, NewPagoProveedorModel,
    UpdatePagoFileModel, UpdatePagoProveedorModel,
};

// ============================================================================
// PAGO FILE REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait PagoFileRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoFileModel>, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Option<PagoFileModel>, ApplicationError>;
    async fn find_by_agencia(&self, id_agencia: i32, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError>;
    async fn find_filtered(&self, id_agencia: Option<i32>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError>;
    async fn count_filtered(&self, id_agencia: Option<i32>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>) -> Result<i64, ApplicationError>;
    async fn create(&self, data: NewPagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError>;
    async fn update(&self, id: i32, data: UpdatePagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError>;
    async fn find_all_by_file(&self, id_file: i32) -> Result<Vec<PagoFileModel>, ApplicationError>;
}

// ============================================================================
// PAGO PROVEEDOR REPOSITORY PORT
// ============================================================================

#[async_trait]
pub trait PagoProveedorRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoProveedorModel>, ApplicationError>;
    async fn find_filtered(&self, tipo_proveedor: Option<&str>, estado: Option<&str>, fecha_desde: Option<DateTime<Utc>>, fecha_hasta: Option<DateTime<Utc>>, limit: i64, offset: i64) -> Result<Vec<PagoProveedorModel>, ApplicationError>;
    async fn count_filtered(&self, tipo_proveedor: Option<&str>, estado: Option<&str>, fecha_desde: Option<DateTime<Utc>>, fecha_hasta: Option<DateTime<Utc>>) -> Result<i64, ApplicationError>;
    async fn create(&self, data: NewPagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError>;
    async fn update(&self, id: i32, data: UpdatePagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError>;
    async fn find_by_file_relation(&self, tipo_proveedor: &str, id_file_vehiculo: Option<i32>, id_file_restaurante: Option<i32>, id_file_guia: Option<i32>) -> Result<Option<PagoProveedorModel>, ApplicationError>;
}