use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Conductor;
use super::{PaginationOptions, PaginatedResult};

#[async_trait]
pub trait ConductorRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Conductor>, ApplicationError>;
    async fn update(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Conductor>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Conductor>, ApplicationError>;
    
    // Específicos de Conductor
    async fn find_by_brevete(&self, nro_brevete: &str) -> Result<Option<Conductor>, ApplicationError>;
    async fn exists_by_brevete(&self, nro_brevete: &str) -> Result<bool, ApplicationError>;
    async fn find_by_transporte(&self, id_transporte: i32) -> Result<Vec<Conductor>, ApplicationError>;
    async fn list_available(&self) -> Result<Vec<Conductor>, ApplicationError>;
    
    /// Actualiza el status del conductor (disponible, ocupado, etc.)
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError>;
}
