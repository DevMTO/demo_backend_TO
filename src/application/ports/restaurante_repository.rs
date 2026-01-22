use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Restaurante;
use crate::application::dtos::RestauranteListItemDto;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait RestauranteRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, restaurante: &Restaurante) -> Result<Restaurante, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Restaurante>, ApplicationError>;
    async fn update(&self, restaurante: &Restaurante) -> Result<Restaurante, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Restaurante>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Restaurante>, ApplicationError>;
    
    /// Lista restaurantes con nombre del encargado (join con personas)
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<RestauranteListItemDto>, i64), ApplicationError>;
    
    // Soft delete
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    // Específicos de Restaurante
    async fn find_by_tipo_atencion(&self, tipo: &str) -> Result<Vec<Restaurante>, ApplicationError>;
    async fn find_by_min_capacity(&self, min_capacity: i32) -> Result<Vec<Restaurante>, ApplicationError>;
}
