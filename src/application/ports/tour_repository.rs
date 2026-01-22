use async_trait::async_trait;
use bigdecimal::BigDecimal;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Tour;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait TourRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, tour: &Tour) -> Result<Tour, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Tour>, ApplicationError>;
    async fn update(&self, tour: &Tour) -> Result<Tour, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tour>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Tour>, ApplicationError>;
    async fn list_all_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Tour>, ApplicationError>;
    
    // Soft delete
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    // Búsqueda
    async fn search(&self, query: &str) -> Result<Vec<Tour>, ApplicationError>;
    
    // Específicos de Tour
    async fn find_by_nombre(&self, nombre: &str) -> Result<Vec<Tour>, ApplicationError>;
    async fn find_by_precio_range(&self, min: BigDecimal, max: BigDecimal) -> Result<Vec<Tour>, ApplicationError>;
    async fn find_by_duracion(&self, dias: i32) -> Result<Vec<Tour>, ApplicationError>;
}
