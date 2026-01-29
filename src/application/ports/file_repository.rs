use async_trait::async_trait;
use chrono::NaiveDate;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::File;
use super::{PaginationOptions, PaginatedResult};

#[async_trait]
pub trait FileRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, file: &File) -> Result<File, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<File>, ApplicationError>;
    async fn update(&self, file: &File) -> Result<File, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    
    // Soft delete y restore
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<File>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<File>, ApplicationError>;
    
    // Específicos de File
    async fn find_by_agencia(&self, id_agencia: i32) -> Result<Vec<File>, ApplicationError>;
    async fn find_by_date_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<File>, ApplicationError>;
    async fn find_upcoming(&self) -> Result<Vec<File>, ApplicationError>;
    async fn find_pending_payment(&self) -> Result<Vec<File>, ApplicationError>;
}
