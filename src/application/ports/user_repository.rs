
use async_trait::async_trait;
use crate::domain::{entities::User, errors::ApplicationError};
use crate::application::dtos::UserListItemDto;

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
    
    async fn list_users_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<UserListItemDto>, i64), ApplicationError>;
}
