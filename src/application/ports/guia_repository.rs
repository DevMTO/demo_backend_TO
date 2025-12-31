use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Guia;
use crate::application::dtos::GuiaListItemDto;
use super::{PaginationOptions, PaginatedResult};

#[async_trait]
pub trait GuiaRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, guia: &Guia) -> Result<Guia, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Guia>, ApplicationError>;
    async fn update(&self, guia: &Guia) -> Result<Guia, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Guia>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    #[allow(dead_code)]
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Guia>, ApplicationError>;
    
    /// Lista guías con información completa de la persona asociada
    async fn list_with_persona(&self, limit: i64, offset: i64) -> Result<(Vec<GuiaListItemDto>, i64), ApplicationError>;
    
    // Específicos de Guia
    async fn find_by_carnet(&self, nro_carnet: &str) -> Result<Option<Guia>, ApplicationError>;
    async fn exists_by_carnet(&self, nro_carnet: &str) -> Result<bool, ApplicationError>;
    async fn list_available(&self) -> Result<Vec<Guia>, ApplicationError>;
    async fn find_by_idioma(&self, idioma: &str) -> Result<Vec<Guia>, ApplicationError>;
    async fn find_by_especialidad(&self, especialidad: &str) -> Result<Vec<Guia>, ApplicationError>;
}
