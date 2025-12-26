//! # Session Repository Port
//! 
//! Puerto de salida para persistencia de sesiones.


use async_trait::async_trait;
use crate::domain::{entities::UserSession, errors::ApplicationError};

/// Puerto de salida para repositorio de sesiones
#[async_trait]
pub trait SessionRepositoryPort: Send + Sync {
    /// Crear una nueva sesión
    async fn create(&self, session: &UserSession) -> Result<UserSession, ApplicationError>;
    
    /// Buscar sesión por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<UserSession>, ApplicationError>;
    
    /// Buscar sesión por token hash
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<UserSession>, ApplicationError>;
    
    /// Buscar sesiones activas por ID de usuario
    async fn find_active_by_user_id(&self, user_id: i32) -> Result<Vec<UserSession>, ApplicationError>;
    
    /// Actualizar sesión
    async fn update(&self, session: &UserSession) -> Result<UserSession, ApplicationError>;
    
    /// Eliminar sesión por ID
    async fn delete(&self, id: i32) -> Result<(), ApplicationError>;
    
    /// Eliminar todas las sesiones de un usuario
    async fn delete_by_user_id(&self, user_id: i32) -> Result<u64, ApplicationError>;
    
    /// Revocar sesión
    async fn revoke(&self, id: i32, reason: &str) -> Result<(), ApplicationError>;
    
    /// Revocar todas las sesiones de un usuario excepto la actual
    async fn revoke_all_except(&self, user_id: i32, except_session_id: i32, reason: &str) -> Result<u64, ApplicationError>;
    
    /// Eliminar sesiones expiradas
    async fn delete_expired(&self) -> Result<u64, ApplicationError>;
    
    /// Contar sesiones activas de un usuario
    async fn count_active_by_user_id(&self, user_id: i32) -> Result<i64, ApplicationError>;
}
