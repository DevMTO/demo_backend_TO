use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Persona;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait PersonaRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, persona: &Persona) -> Result<Persona, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Persona>, ApplicationError>;
    async fn update(&self, persona: &Persona) -> Result<Persona, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Persona>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Persona>, ApplicationError>;
    
    // Específicos de Persona
    async fn find_by_documento(&self, tipo: &str, numero: &str) -> Result<Option<Persona>, ApplicationError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Persona>, ApplicationError>;
    async fn exists_by_documento(&self, tipo: &str, numero: &str) -> Result<bool, ApplicationError>;
    async fn search(&self, query: &str) -> Result<Vec<Persona>, ApplicationError>;
}
