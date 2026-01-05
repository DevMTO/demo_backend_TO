use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Transporte;
use crate::application::dtos::TransporteListItemDto;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait TransporteRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, transporte: &Transporte) -> Result<Transporte, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Transporte>, ApplicationError>;
    async fn update(&self, transporte: &Transporte) -> Result<Transporte, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Transporte>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Transporte>, ApplicationError>;
    
    // List con encargado
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<TransporteListItemDto>, i64), ApplicationError>;
    
    // Soft delete
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    // Específicos de Transporte
    async fn find_by_ruc(&self, ruc: &str) -> Result<Option<Transporte>, ApplicationError>;
    async fn exists_by_ruc(&self, ruc: &str) -> Result<bool, ApplicationError>;
    async fn find_with_available_vehicles(&self) -> Result<Vec<Transporte>, ApplicationError>;
    
    /// Busca un transporte por el ID de la persona encargada
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<Transporte>, ApplicationError>;
}
