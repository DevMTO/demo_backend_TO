use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Conductor;
use crate::application::dtos::ConductorListItemDto;

#[async_trait]
pub trait ConductorRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Conductor>, ApplicationError>;
    async fn update(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    
    // Soft delete y restore
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;

    // Listado con detalles de persona y transporte
    async fn list_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<ConductorListItemDto>, i64), ApplicationError>;
    
    // Específicos de Conductor
    async fn find_by_brevete(&self, nro_brevete: &str) -> Result<Option<Conductor>, ApplicationError>;
    async fn exists_by_brevete(&self, nro_brevete: &str) -> Result<bool, ApplicationError>;
    async fn find_by_transporte(&self, id_transporte: i32) -> Result<Vec<Conductor>, ApplicationError>;
    async fn list_available(&self) -> Result<Vec<Conductor>, ApplicationError>;
    
    /// Actualiza el status del conductor (disponible, ocupado, etc.)
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError>;
    
    /// Listar conductores de un transporte con paginación
    async fn list_by_transporte_paginated(&self, transporte_id: i32, limit: i64, offset: i64) -> Result<(Vec<ConductorListItemDto>, i64), ApplicationError>;
    
    /// Listar conductores disponibles de un transporte específico
    async fn list_available_by_transporte(&self, transporte_id: i32) -> Result<Vec<Conductor>, ApplicationError>;
}
