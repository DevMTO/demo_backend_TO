use async_trait::async_trait;
use bigdecimal::BigDecimal;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Pago;
use super::{PaginationOptions, PaginatedResult};

#[async_trait]
pub trait PagoRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, pago: &Pago) -> Result<Pago, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Pago>, ApplicationError>;
    async fn update(&self, pago: &Pago) -> Result<Pago, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Pago>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Pago>, ApplicationError>;
    
    // Específicos de Pago
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<Pago>, ApplicationError>;
    async fn sum_ingresos_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError>;
    async fn sum_egresos_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError>;
    async fn get_balance_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError>;
}
