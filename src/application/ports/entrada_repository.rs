use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Entrada;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait EntradaRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, entrada: &Entrada) -> Result<Entrada, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Entrada>, ApplicationError>;
    async fn update(&self, entrada: &Entrada) -> Result<Entrada, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Entrada>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Entrada>, ApplicationError>;
    
    // Soft delete
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    // Búsqueda por ruta
    async fn find_by_ruta(&self, ruta: &str) -> Result<Vec<Entrada>, ApplicationError>;
}
