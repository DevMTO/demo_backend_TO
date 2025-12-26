//! # User Repository Port
//! 
//! Puerto de salida para persistencia de usuarios.


use async_trait::async_trait;
use crate::domain::{entities::User, errors::ApplicationError};

/// Puerto de salida para repositorio de usuarios
#[async_trait]
pub trait UserRepositoryPort: Send + Sync {
    /// Crear un nuevo usuario
    async fn create(&self, user: &User) -> Result<User, ApplicationError>;
    
    /// Buscar usuario por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, ApplicationError>;
    
    /// Buscar usuario por email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError>;
    
    /// Buscar usuario por username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApplicationError>;
    
    /// Buscar usuario por email o username
    async fn find_by_email_or_username(&self, identifier: &str) -> Result<Option<User>, ApplicationError>;
    
    /// Actualizar usuario
    async fn update(&self, user: &User) -> Result<User, ApplicationError>;
    
    /// Eliminar usuario (soft delete)
    async fn delete(&self, id: i32) -> Result<(), ApplicationError>;
    
    /// Verificar si existe un email
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError>;
    
    /// Verificar si existe un username
    async fn exists_by_username(&self, username: &str) -> Result<bool, ApplicationError>;
    
    /// Listar usuarios activos
    async fn list_active(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>, ApplicationError>;
    
    /// Contar usuarios activos
    async fn count_active(&self) -> Result<i64, ApplicationError>;
}
