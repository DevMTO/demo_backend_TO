//! # Async Session Repository Implementation
//! 
//! Implementación asíncrona del puerto SessionRepositoryPort usando diesel-async.

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::application::ports::SessionRepositoryPort;
use crate::domain::{entities::UserSession, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{NewSessionModel, SessionModel},
    schema::user_sessions,
};

/// Implementación asíncrona del repositorio de sesiones
pub struct PostgresSessionRepository {
    pool: DatabasePool,
}

impl PostgresSessionRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepositoryPort for PostgresSessionRepository {
    async fn create(&self, session: &UserSession) -> Result<UserSession, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_session: NewSessionModel = session.into();
        
        let result = diesel::insert_into(user_sessions::table)
            .values(&new_session)
            .get_result::<SessionModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserSession>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = user_sessions::table
            .filter(user_sessions::id.eq(id))
            .first::<SessionModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<UserSession>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = user_sessions::table
            .filter(user_sessions::token_hash.eq(token_hash))
            .filter(user_sessions::is_active.eq(true))
            .first::<SessionModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn find_active_by_user_id(&self, user_id: &Uuid) -> Result<Vec<UserSession>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::is_active.eq(true))
            .filter(user_sessions::expires_at.gt(Utc::now()))
            .order(user_sessions::created_at.asc())
            .load::<SessionModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn update(&self, session: &UserSession) -> Result<UserSession, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(user_sessions::table.filter(user_sessions::id.eq(&session.id)))
            .set((
                user_sessions::token_hash.eq(&session.token_hash),
                user_sessions::refresh_token_hash.eq(&session.refresh_token_hash),
                user_sessions::expires_at.eq(session.expires_at),
                user_sessions::refresh_expires_at.eq(session.refresh_expires_at),
                user_sessions::updated_at.eq(Utc::now()),
                user_sessions::is_active.eq(session.is_active),
                user_sessions::revoked_at.eq(session.revoked_at),
                user_sessions::revoked_reason.eq(&session.revoked_reason),
            ))
            .get_result::<SessionModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::delete(user_sessions::table.filter(user_sessions::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete_by_user_id(&self, user_id: &Uuid) -> Result<u64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = diesel::delete(user_sessions::table.filter(user_sessions::user_id.eq(user_id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count as u64)
    }
    
    async fn revoke(&self, id: &Uuid, reason: &str) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::update(user_sessions::table.filter(user_sessions::id.eq(id)))
            .set((
                user_sessions::is_active.eq(false),
                user_sessions::revoked_at.eq(Some(Utc::now())),
                user_sessions::revoked_reason.eq(Some(reason)),
                user_sessions::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(())
    }
    
    async fn revoke_all_except(&self, user_id: &Uuid, except_session_id: &Uuid, reason: &str) -> Result<u64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = diesel::update(
            user_sessions::table
                .filter(user_sessions::user_id.eq(user_id))
                .filter(user_sessions::id.ne(except_session_id))
                .filter(user_sessions::is_active.eq(true))
        )
        .set((
            user_sessions::is_active.eq(false),
            user_sessions::revoked_at.eq(Some(Utc::now())),
            user_sessions::revoked_reason.eq(Some(reason)),
            user_sessions::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count as u64)
    }
    
    async fn delete_expired(&self) -> Result<u64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = diesel::delete(
            user_sessions::table.filter(user_sessions::expires_at.lt(Utc::now()))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count as u64)
    }
    
    async fn count_active_by_user_id(&self, user_id: &Uuid) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::is_active.eq(true))
            .filter(user_sessions::expires_at.gt(Utc::now()))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count)
    }
}
