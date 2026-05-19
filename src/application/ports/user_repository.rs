
use async_trait::async_trait;
use crate::domain::{entities::User, errors::ApplicationError};
use crate::application::dtos::UserListItemDto;

/// Scope restriction for listing users based on the authenticated user's role
#[derive(Debug)]
pub enum UserListScope {
    /// No restriction — SuperAdmin and Admin see all users
    All,
    /// Restrict to users belonging to a specific agencia
    AgenciaScope { id_entidad: i32 },
    /// Restrict to hoteles_gerente_cadena of this cadena, hoteles_gerente of its hotels,
    /// and hoteles users of its hotels
    HotelCadenaScope { id_cadena: i32 },
    /// Restrict to hoteles_gerente and hoteles users of a specific hotel
    HotelScope { id_hotel: i32 },
    /// Empty result - for gerentes with invalid/missing id_entidad (security)
    Empty,
}

#[allow(dead_code)]
#[async_trait]
pub trait UserRepositoryPort: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, ApplicationError>;
    
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, ApplicationError>;
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError>;
    
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApplicationError>;
    
    async fn find_by_email_or_username(&self, identifier: &str) -> Result<Option<User>, ApplicationError>;
    
    async fn update(&self, user: &User) -> Result<User, ApplicationError>;
    
    async fn delete(&self, id: i32) -> Result<(), ApplicationError>;
    
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<(), ApplicationError>;
    
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError>;
    
    async fn exists_by_username(&self, username: &str) -> Result<bool, ApplicationError>;
    
    async fn list_active(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>, ApplicationError>;
    
    async fn count_active(&self) -> Result<i64, ApplicationError>;
    
    async fn list_users_with_details(&self, limit: i64, offset: i64, is_demo: Option<bool>, scope: &UserListScope) -> Result<(Vec<UserListItemDto>, i64), ApplicationError>;
    
    /// Encuentra usuarios por rol e id_entidad (para notificaciones a proveedores)
    async fn find_by_role_and_entity(&self, role: &str, entity_id: i32) -> Result<Vec<User>, ApplicationError>;
    
    /// Encuentra usuarios por id_persona
    async fn find_by_persona_id(&self, persona_id: i32) -> Result<Vec<User>, ApplicationError>;
}
