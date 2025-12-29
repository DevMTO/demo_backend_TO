use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Agencia;
use crate::application::dtos::AgenciaListItemDto;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait AgenciaRepositoryPort: Send + Sync {
    async fn create(&self, agencia: &Agencia) -> Result<Agencia, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Agencia>, ApplicationError>;
    async fn update(&self, agencia: &Agencia) -> Result<Agencia, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Agencia>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Agencia>, ApplicationError>;
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    async fn find_by_ruc(&self, ruc: &str) -> Result<Option<Agencia>, ApplicationError>;
    async fn exists_by_ruc(&self, ruc: &str) -> Result<bool, ApplicationError>;
    
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<AgenciaListItemDto>, i64), ApplicationError>;
}
