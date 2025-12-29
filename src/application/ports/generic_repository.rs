use async_trait::async_trait;
use crate::domain::errors::ApplicationError;

#[derive(Debug, Clone, Default)]
pub struct PaginationOptions {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[allow(dead_code)]
impl PaginationOptions {
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self { limit, offset }
    }
    
    /// Crear con límite por defecto (50)
    pub fn with_defaults() -> Self {
        Self { limit: Some(50), offset: Some(0) }
    }
}

#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self { data, total, limit, offset }
    }
    
    pub fn pages(&self) -> i64 {
        if self.limit == 0 { return 1; }
        (self.total + self.limit - 1) / self.limit
    }
    
    pub fn current_page(&self) -> i64 {
        if self.limit == 0 { return 1; }
        self.offset / self.limit + 1
    }
}

#[allow(dead_code)]
#[async_trait]
pub trait GenericRepository<T, ID>: Send + Sync 
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// Crear una nueva entidad
    async fn create(&self, entity: &T) -> Result<T, ApplicationError>;
    
    /// Buscar entidad por ID
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, ApplicationError>;
    
    /// Actualizar entidad existente
    async fn update(&self, entity: &T) -> Result<T, ApplicationError>;
    
    /// Eliminar entidad por ID (soft delete cuando aplica)
    async fn delete(&self, id: ID) -> Result<bool, ApplicationError>;
    
    /// Listar todas las entidades activas con paginación
    async fn list(&self, pagination: PaginationOptions) -> Result<Vec<T>, ApplicationError>;
    
    /// Contar total de entidades activas
    async fn count(&self) -> Result<i64, ApplicationError>;
    
    /// Listar con paginación y retornar resultado paginado
    async fn list_paginated(&self, pagination: PaginationOptions) -> Result<PaginatedResult<T>, ApplicationError> {
        let total = self.count().await?;
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        let data = self.list(pagination).await?;
        
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
}

#[allow(dead_code)]
#[async_trait]
pub trait SoftDeleteRepository<T, ID>: GenericRepository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync + Clone,
{
    /// Soft delete (marcar como inactivo)
    async fn soft_delete(&self, id: ID) -> Result<bool, ApplicationError>;
    
    /// Restaurar entidad soft-deleted
    async fn restore(&self, id: ID) -> Result<bool, ApplicationError>;
    
    /// Listar incluyendo inactivos
    async fn list_all(&self, pagination: PaginationOptions) -> Result<Vec<T>, ApplicationError>;
}

#[allow(dead_code)]
#[async_trait]
pub trait SearchableRepository<T, ID>: GenericRepository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// Buscar por nombre (like case-insensitive)
    async fn search_by_name(&self, query: &str, pagination: PaginationOptions) -> Result<Vec<T>, ApplicationError>;
}
