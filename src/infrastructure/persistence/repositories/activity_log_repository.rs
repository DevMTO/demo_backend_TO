use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, instrument};

use crate::application::ports::{
    ActivityLogRepositoryPort, 
    ActivityLogFilters as PortFilters,
    CountByType,
    PaginationOptions, 
    PaginatedResult,
};
use crate::domain::{entities::{ActivityLog, NewActivityLog}, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{ActivityLogModel, NewActivityLogModel},
    schema::activity_logs,
};

pub struct PostgresActivityLogRepository {
    pool: DatabasePool,
}

impl PostgresActivityLogRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActivityLogRepositoryPort for PostgresActivityLogRepository {
    #[instrument(skip(self, log))]
    async fn create(&self, log: NewActivityLog) -> Result<ActivityLog, ApplicationError> {
        debug!("Creando log de actividad: {} - {}", log.action_type, log.action);
        let mut conn = self.pool.get_connection().await?;
        
        let new_log: NewActivityLogModel = log.into();
        
        let result = diesel::insert_into(activity_logs::table)
            .values(&new_log)
            .get_result::<ActivityLogModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al crear log: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        debug!("Log creado con ID: {}", result.id);
        Ok(result.into())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<ActivityLog>, ApplicationError> {
        debug!("Buscando log por ID: {}", id);
        let mut conn = self.pool.get_connection().await?;
        
        let result = activity_logs::table
            .filter(activity_logs::id.eq(id))
            .first::<ActivityLogModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("Error al buscar log: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        Ok(result.map(Into::into))
    }

    #[instrument(skip(self, filters, pagination))]
    async fn find_all(
        &self,
        filters: PortFilters,
        pagination: PaginationOptions,
    ) -> Result<PaginatedResult<ActivityLog>, ApplicationError> {
        debug!("Listando logs con filtros");
        let mut conn = self.pool.get_connection().await?;
        
        // Query base
        let mut query = activity_logs::table.into_boxed();
        
        // Aplicar filtros
        if let Some(user_id) = filters.user_id {
            query = query.filter(activity_logs::user_id.eq(user_id));
        }
        if let Some(action_type) = &filters.action_type {
            query = query.filter(activity_logs::action_type.eq(action_type));
        }
        if let Some(action) = &filters.action {
            query = query.filter(activity_logs::action.eq(action));
        }
        if let Some(entity_type) = &filters.entity_type {
            query = query.filter(activity_logs::entity_type.eq(entity_type));
        }
        if let Some(entity_id) = filters.entity_id {
            query = query.filter(activity_logs::entity_id.eq(entity_id));
        }
        if let Some(status) = &filters.status {
            query = query.filter(activity_logs::status.eq(status));
        }
        if let Some(from_date) = filters.from_date {
            query = query.filter(activity_logs::created_at.ge(from_date));
        }
        if let Some(to_date) = filters.to_date {
            query = query.filter(activity_logs::created_at.le(to_date));
        }
        
        // Contar total
        let count_query = activity_logs::table.into_boxed();
        let mut count_query = count_query;
        
        if let Some(user_id) = filters.user_id {
            count_query = count_query.filter(activity_logs::user_id.eq(user_id));
        }
        if let Some(action_type) = &filters.action_type {
            count_query = count_query.filter(activity_logs::action_type.eq(action_type));
        }
        if let Some(action) = &filters.action {
            count_query = count_query.filter(activity_logs::action.eq(action));
        }
        if let Some(entity_type) = &filters.entity_type {
            count_query = count_query.filter(activity_logs::entity_type.eq(entity_type));
        }
        if let Some(entity_id) = filters.entity_id {
            count_query = count_query.filter(activity_logs::entity_id.eq(entity_id));
        }
        if let Some(status) = &filters.status {
            count_query = count_query.filter(activity_logs::status.eq(status));
        }
        if let Some(from_date) = filters.from_date {
            count_query = count_query.filter(activity_logs::created_at.ge(from_date));
        }
        if let Some(to_date) = filters.to_date {
            count_query = count_query.filter(activity_logs::created_at.le(to_date));
        }
        
        let total: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        // Aplicar paginación y ordenar
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        
        let logs: Vec<ActivityLogModel> = query
            .order(activity_logs::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(PaginatedResult {
            data: logs.into_iter().map(Into::into).collect(),
            total,
            limit,
            offset,
        })
    }

    #[instrument(skip(self, filters))]
    async fn count(&self, filters: PortFilters) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = activity_logs::table.into_boxed();
        
        if let Some(user_id) = filters.user_id {
            query = query.filter(activity_logs::user_id.eq(user_id));
        }
        if let Some(action_type) = &filters.action_type {
            query = query.filter(activity_logs::action_type.eq(action_type));
        }
        if let Some(action) = &filters.action {
            query = query.filter(activity_logs::action.eq(action));
        }
        if let Some(entity_type) = &filters.entity_type {
            query = query.filter(activity_logs::entity_type.eq(entity_type));
        }
        if let Some(entity_id) = filters.entity_id {
            query = query.filter(activity_logs::entity_id.eq(entity_id));
        }
        if let Some(status) = &filters.status {
            query = query.filter(activity_logs::status.eq(status));
        }
        if let Some(from_date) = filters.from_date {
            query = query.filter(activity_logs::created_at.ge(from_date));
        }
        if let Some(to_date) = filters.to_date {
            query = query.filter(activity_logs::created_at.le(to_date));
        }
        
        let total: i64 = query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(total)
    }

    #[instrument(skip(self))]
    async fn count_by_action_type(&self) -> Result<Vec<CountByType>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results: Vec<(String, i64)> = activity_logs::table
            .group_by(activity_logs::action_type)
            .select((activity_logs::action_type, diesel::dsl::count_star()))
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results
            .into_iter()
            .map(|(key, count)| CountByType { key, count })
            .collect())
    }

    #[instrument(skip(self))]
    async fn count_by_status(&self) -> Result<Vec<CountByType>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results: Vec<(String, i64)> = activity_logs::table
            .group_by(activity_logs::status)
            .select((activity_logs::status, diesel::dsl::count_star()))
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results
            .into_iter()
            .map(|(key, count)| CountByType { key, count })
            .collect())
    }

    #[instrument(skip(self))]
    async fn find_recent_errors(&self, limit: i64) -> Result<Vec<ActivityLog>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let logs: Vec<ActivityLogModel> = activity_logs::table
            .filter(activity_logs::status.eq("error"))
            .order(activity_logs::created_at.desc())
            .limit(limit)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(logs.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self))]
    async fn cleanup_old_logs(&self, older_than_days: i64) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let before = chrono::Utc::now() - chrono::Duration::days(older_than_days);
        
        let deleted = diesel::delete(
            activity_logs::table.filter(activity_logs::created_at.lt(before))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        debug!("🗑️ Eliminados {} logs antiguos", deleted);
        Ok(deleted as i64)
    }
}
