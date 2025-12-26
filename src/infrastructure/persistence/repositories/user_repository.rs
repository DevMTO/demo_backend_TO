//! # Async User Repository Implementation
//! 
//! Implementación asíncrona del puerto UserRepositoryPort usando diesel-async.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::application::ports::UserRepositoryPort;
use crate::domain::{entities::User, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{NewUserModel, UserModel},
    schema::users,
};

/// Implementación asíncrona del repositorio de usuarios
pub struct PostgresUserRepository {
    pool: DatabasePool,
}

impl PostgresUserRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryPort for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<User, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_user: NewUserModel = user.into();
        
        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::id.eq(id))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::email.eq(email.to_lowercase()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::username.eq(username))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn find_by_email_or_username(&self, identifier: &str) -> Result<Option<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let identifier_lower = identifier.to_lowercase();
        
        let result = users::table
            .filter(
                users::email.eq(&identifier_lower)
                    .or(users::username.eq(identifier))
            )
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, user: &User) -> Result<User, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(users::table.filter(users::id.eq(&user.id)))
            .set((
                users::username.eq(&user.username),
                users::email.eq(&user.email),
                users::password_hash.eq(&user.password_hash),
                users::display_name.eq(&user.display_name),
                users::role.eq(user.role.to_string()),
                users::email_verified.eq(user.email_verified),
                users::is_active.eq(user.is_active),
                users::updated_at.eq(user.updated_at),
                users::last_login.eq(user.last_login),
                users::updated_by.eq(&user.updated_by),
                users::version.eq(user.version),
                users::mfa_enabled.eq(user.mfa_enabled),
                users::mfa_secret.eq(&user.mfa_secret),
                users::mfa_backup_codes.eq(&user.mfa_backup_codes),
            ))
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(())
    }
    
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = users::table
            .filter(users::email.eq(email.to_lowercase()))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    async fn exists_by_username(&self, username: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = users::table
            .filter(users::username.eq(username))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    async fn list_active(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = users::table
            .filter(users::is_active.eq(true))
            .order(users::created_at.desc())
            .into_boxed();
        
        if let Some(l) = limit {
            query = query.limit(l);
        }
        
        if let Some(o) = offset {
            query = query.offset(o);
        }
        
        let results = query
            .load::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count_active(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = users::table
            .filter(users::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count)
    }
}
