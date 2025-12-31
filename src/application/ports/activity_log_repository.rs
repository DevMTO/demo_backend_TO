use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::entities::{ActivityLog, NewActivityLog};
use crate::domain::errors::ApplicationError;
use super::generic_repository::{PaginationOptions, PaginatedResult};

#[derive(Debug, Clone, Default)]
pub struct ActivityLogFilters {
    pub user_id: Option<i32>,
    pub action_type: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CountByType {
    pub key: String,
    pub count: i64,
}

#[async_trait]
pub trait ActivityLogRepositoryPort: Send + Sync {
    /// Crear nuevo log de actividad
    #[allow(dead_code)]
    async fn create(&self, log: NewActivityLog) -> Result<ActivityLog, ApplicationError>;
    
    /// Obtener log por ID
    #[allow(dead_code)]
    async fn find_by_id(&self, id: i32) -> Result<Option<ActivityLog>, ApplicationError>;
    
    /// Listar logs con filtros y paginación
    async fn find_all(
        &self, 
        filters: ActivityLogFilters,
        pagination: PaginationOptions
    ) -> Result<PaginatedResult<ActivityLog>, ApplicationError>;
    
    /// Contar logs con filtros
    async fn count(&self, filters: ActivityLogFilters) -> Result<i64, ApplicationError>;
    
    /// Contar logs por tipo de acción
    async fn count_by_action_type(&self) -> Result<Vec<CountByType>, ApplicationError>;
    
    /// Contar logs por status
    async fn count_by_status(&self) -> Result<Vec<CountByType>, ApplicationError>;
    
    /// Obtener logs de error recientes
    async fn find_recent_errors(&self, limit: i64) -> Result<Vec<ActivityLog>, ApplicationError>;
    
    /// Limpiar logs antiguos (para mantenimiento)
    async fn cleanup_old_logs(&self, older_than_days: i64) -> Result<i64, ApplicationError>;
}
