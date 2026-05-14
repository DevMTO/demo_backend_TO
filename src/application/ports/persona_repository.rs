use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Persona;
use super::{PaginationOptions, PaginatedResult};

/// Scope restriction for listing personas based on authenticated user's role
#[derive(Debug)]
pub enum PersonaListScope {
    /// No restriction — SuperAdmin, Admin see all personas
    All,
    /// Gerentes see personas they created OR personas with users in their entity
    GerenteScope {
        created_by_user_id: i32,
        id_entidad: i32,
    },
    /// Empty result - for gerentes with invalid/missing id_entidad (security)
    Empty,
}

#[allow(dead_code)]
#[async_trait]
pub trait PersonaRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, persona: &Persona) -> Result<Persona, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Persona>, ApplicationError>;
    async fn update(&self, persona: &Persona) -> Result<Persona, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Persona>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Persona>, ApplicationError>;
    
    /// List personas with scope-based filtering (for gerente users)
    async fn list_paginated_with_scope(
        &self,
        options: PaginationOptions,
        scope: &PersonaListScope,
    ) -> Result<PaginatedResult<Persona>, ApplicationError>;

    // Específicos de Persona
    async fn find_by_documento(&self, tipo: &str, numero: &str) -> Result<Option<Persona>, ApplicationError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Persona>, ApplicationError>;
    async fn exists_by_documento(&self, tipo: &str, numero: &str) -> Result<bool, ApplicationError>;
    async fn search(&self, query: &str, scope: &PersonaListScope) -> Result<Vec<Persona>, ApplicationError>;
}
