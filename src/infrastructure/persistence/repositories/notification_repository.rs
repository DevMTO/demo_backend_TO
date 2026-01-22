use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, info, instrument};

use crate::application::ports::{
    NotificationRepositoryPort,
    NotificationFilters as PortFilters,
    PriorityCount,
    PaginationOptions,
    PaginatedResult,
};
use crate::domain::{
    entities::{
        Notification, NewNotification,
        NotificationUser,
        NotificationWithReadStatus,
    },
    errors::ApplicationError,
};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{
        NotificationModel, NewNotificationModel,
        NotificationUserModel, NewNotificationUserModel,
        NotificationWithUserData,
    },
    schema::{notifications, notification_users, users},
};

/// Resultado de la función cleanup_notifications de PostgreSQL
#[derive(QueryableByName, Debug)]
struct CleanupNotificationsResult {
    #[diesel(sql_type = BigInt)]
    deleted_expired: i64,
    #[diesel(sql_type = BigInt)]
    deleted_low: i64,
    #[diesel(sql_type = BigInt)]
    deleted_normal: i64,
    #[diesel(sql_type = BigInt)]
    deleted_high: i64,
    #[diesel(sql_type = BigInt)]
    deleted_urgent: i64,
    #[diesel(sql_type = BigInt)]
    total_deleted: i64,
}

pub struct PostgresNotificationRepository {
    pool: DatabasePool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepositoryPort for PostgresNotificationRepository {
    // ===== Notificaciones =====

    #[instrument(skip(self, notification))]
    async fn create(&self, notification: NewNotification) -> Result<Notification, ApplicationError> {
        debug!("Creando notificación: {}", notification.title);
        let mut conn = self.pool.get_connection().await?;
        
        let new_notification: NewNotificationModel = notification.into();
        
        let result = diesel::insert_into(notifications::table)
            .values(&new_notification)
            .get_result::<NotificationModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al crear notificación: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("Notificación creada con ID: {}", result.id);
        Ok(result.into())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<Notification>, ApplicationError> {
        debug!("Buscando notificación por ID: {}", id);
        let mut conn = self.pool.get_connection().await?;
        
        let result = notifications::table
            .filter(notifications::id.eq(id))
            .first::<NotificationModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("Error al buscar notificación: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        Ok(result.map(Into::into))
    }

    #[instrument(skip(self, filters))]
    async fn find_all(
        &self,
        filters: PortFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>, ApplicationError> {
        debug!("Listando todas las notificaciones");
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = notifications::table.into_boxed();
        
        if let Some(notification_type) = &filters.notification_type {
            query = query.filter(notifications::notification_type.eq(notification_type));
        }
        if let Some(category) = &filters.category {
            query = query.filter(notifications::category.eq(category));
        }
        if let Some(priority) = &filters.priority {
            query = query.filter(notifications::priority.eq(priority));
        }
        
        let items: Vec<NotificationModel> = query
            .order(notifications::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(items.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self, filters))]
    async fn count(&self, filters: PortFilters) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = notifications::table.into_boxed();
        
        if let Some(notification_type) = &filters.notification_type {
            query = query.filter(notifications::notification_type.eq(notification_type));
        }
        if let Some(category) = &filters.category {
            query = query.filter(notifications::category.eq(category));
        }
        if let Some(priority) = &filters.priority {
            query = query.filter(notifications::priority.eq(priority));
        }
        
        let total: i64 = query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(total)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        debug!("🗑️ Eliminando notificación: {}", id);
        let mut conn = self.pool.get_connection().await?;
        
        let deleted = diesel::delete(notifications::table.filter(notifications::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(deleted > 0)
    }

    #[instrument(skip(self))]
    async fn cleanup_expired(&self) -> Result<i64, ApplicationError> {
        debug!("🗑️ Limpiando notificaciones expiradas");
        let mut conn = self.pool.get_connection().await?;
        let now = Utc::now();
        
        let deleted = diesel::delete(
            notifications::table.filter(
                notifications::expires_at.is_not_null()
                    .and(notifications::expires_at.lt(now))
            )
        )
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Eliminadas {} notificaciones expiradas", deleted);
        Ok(deleted as i64)
    }

    #[instrument(skip(self))]
    async fn cleanup_by_priority(
        &self,
        days_low: i32,
        days_normal: i32,
        days_high: i32,
        days_urgent: i32,
    ) -> Result<crate::application::ports::CleanupResult, ApplicationError> {
        use diesel::sql_types::Integer;
        
        debug!("🗑️ Ejecutando cleanup de notificaciones por prioridad");
        let mut conn = self.pool.get_connection().await?;
        
        // Ejecutar la función de cleanup que creamos en la migración
        let result: (i64, i64, i64, i64, i64, i64) = diesel::sql_query(
            "SELECT * FROM cleanup_notifications($1, $2, $3, $4)"
        )
        .bind::<Integer, _>(days_low)
        .bind::<Integer, _>(days_normal)
        .bind::<Integer, _>(days_high)
        .bind::<Integer, _>(days_urgent)
        .get_result::<CleanupNotificationsResult>(&mut conn)
        .await
        .map(|r| (r.deleted_expired, r.deleted_low, r.deleted_normal, r.deleted_high, r.deleted_urgent, r.total_deleted))
        .map_err(|e| {
            warn!("Error en cleanup_by_priority: {}", e);
            ApplicationError::Repository(e.to_string())
        })?;
        
        info!("Cleanup completado - Total eliminadas: {}", result.5);
        
        Ok(crate::application::ports::CleanupResult {
            deleted_expired: result.0,
            deleted_low: result.1,
            deleted_normal: result.2,
            deleted_high: result.3,
            deleted_urgent: result.4,
            total_deleted: result.5,
        })
    }

    // ===== Notificaciones de Usuario =====

    #[instrument(skip(self))]
    async fn create_user_notification(
        &self,
        notification_id: i32,
        user_id: i32,
    ) -> Result<NotificationUser, ApplicationError> {
        debug!("Creando notificación de usuario: notification={}, user={}", notification_id, user_id);
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewNotificationUserModel {
            notification_id,
            user_id,
        };
        
        let result = diesel::insert_into(notification_users::table)
            .values(&new_record)
            .get_result::<NotificationUserModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al crear notificación de usuario: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        Ok(result.into())
    }

    #[instrument(skip(self, user_ids))]
    async fn create_user_notifications_batch(
        &self,
        notification_id: i32,
        user_ids: Vec<i32>,
    ) -> Result<Vec<NotificationUser>, ApplicationError> {
        debug!("Creando notificaciones para {} usuarios", user_ids.len());
        let mut conn = self.pool.get_connection().await?;
        
        let new_records: Vec<NewNotificationUserModel> = user_ids
            .into_iter()
            .map(|user_id| NewNotificationUserModel {
                notification_id,
                user_id,
            })
            .collect();
        
        let results = diesel::insert_into(notification_users::table)
            .values(&new_records)
            .on_conflict_do_nothing() // Evitar duplicados
            .get_results::<NotificationUserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self, filters, pagination))]
    async fn find_user_notifications(
        &self,
        user_id: i32,
        filters: PortFilters,
        pagination: PaginationOptions,
    ) -> Result<PaginatedResult<NotificationWithReadStatus>, ApplicationError> {
        debug!("Listando notificaciones de usuario: {}", user_id);
        let mut conn = self.pool.get_connection().await?;
        
        // Base query con join
        let mut query = notification_users::table
            .inner_join(notifications::table)
            .filter(notification_users::user_id.eq(user_id))
            .into_boxed();
        
        // Aplicar filtros
        if let Some(notification_type) = &filters.notification_type {
            query = query.filter(notifications::notification_type.eq(notification_type));
        }
        if let Some(category) = &filters.category {
            query = query.filter(notifications::category.eq(category));
        }
        if let Some(priority) = &filters.priority {
            query = query.filter(notifications::priority.eq(priority));
        }
        if let Some(is_read) = filters.is_read {
            query = query.filter(notification_users::is_read.eq(is_read));
        }
        if let Some(is_dismissed) = filters.is_dismissed {
            query = query.filter(notification_users::is_dismissed.eq(is_dismissed));
        }
        
        // Contar total con los mismos filtros
        let mut count_query = notification_users::table
            .inner_join(notifications::table)
            .filter(notification_users::user_id.eq(user_id))
            .into_boxed();
        
        if let Some(notification_type) = &filters.notification_type {
            count_query = count_query.filter(notifications::notification_type.eq(notification_type));
        }
        if let Some(category) = &filters.category {
            count_query = count_query.filter(notifications::category.eq(category));
        }
        if let Some(priority) = &filters.priority {
            count_query = count_query.filter(notifications::priority.eq(priority));
        }
        if let Some(is_read) = filters.is_read {
            count_query = count_query.filter(notification_users::is_read.eq(is_read));
        }
        if let Some(is_dismissed) = filters.is_dismissed {
            count_query = count_query.filter(notification_users::is_dismissed.eq(is_dismissed));
        }
        
        let total: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let limit = pagination.limit.unwrap_or(50);
        let offset = pagination.offset.unwrap_or(0);
        
        let items: Vec<NotificationWithUserData> = query
            .select((
                notifications::id,
                notifications::notification_type,
                notifications::category,
                notifications::title,
                notifications::message,
                notifications::entity_type,
                notifications::entity_id,
                notifications::metadata,
                notifications::priority,
                notifications::target_roles,
                notifications::target_user_id,
                notifications::expires_at,
                notifications::created_at,
                notifications::created_by,
                notification_users::is_read,
                notification_users::read_at,
                notification_users::is_dismissed,
            ))
            .order(notifications::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(PaginatedResult {
            data: items.into_iter().map(Into::into).collect(),
            total,
            limit,
            offset,
        })
    }

    #[instrument(skip(self))]
    async fn count_user_notifications(
        &self,
        user_id: i32,
        unread_only: bool,
    ) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = notification_users::table
            .filter(notification_users::user_id.eq(user_id))
            .filter(notification_users::is_dismissed.eq(false))
            .into_boxed();
        
        if unread_only {
            query = query.filter(notification_users::is_read.eq(false));
        }
        
        let total: i64 = query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(total)
    }

    #[instrument(skip(self))]
    async fn count_unread(&self, user_id: i32) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = notification_users::table
            .filter(notification_users::user_id.eq(user_id))
            .filter(notification_users::is_read.eq(false))
            .filter(notification_users::is_dismissed.eq(false))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count)
    }

    #[instrument(skip(self))]
    async fn count_unread_by_priority(&self, user_id: i32) -> Result<Vec<PriorityCount>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results: Vec<(String, i64)> = notification_users::table
            .inner_join(notifications::table)
            .filter(notification_users::user_id.eq(user_id))
            .filter(notification_users::is_read.eq(false))
            .filter(notification_users::is_dismissed.eq(false))
            .group_by(notifications::priority)
            .select((notifications::priority, diesel::dsl::count_star()))
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results
            .into_iter()
            .map(|(priority, count)| PriorityCount { priority, count })
            .collect())
    }

    #[instrument(skip(self, notification_ids))]
    async fn mark_as_read(&self, user_id: i32, notification_ids: Vec<i32>) -> Result<i64, ApplicationError> {
        debug!("Marcando {} notificaciones como leídas", notification_ids.len());
        let mut conn = self.pool.get_connection().await?;
        let now = Utc::now();
        
        let updated = diesel::update(
            notification_users::table
                .filter(notification_users::user_id.eq(user_id))
                .filter(notification_users::notification_id.eq_any(&notification_ids))
        )
        .set((
            notification_users::is_read.eq(true),
            notification_users::read_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(updated as i64)
    }

    #[instrument(skip(self))]
    async fn mark_all_as_read(&self, user_id: i32) -> Result<i64, ApplicationError> {
        debug!("Marcando todas las notificaciones como leídas para usuario: {}", user_id);
        let mut conn = self.pool.get_connection().await?;
        let now = Utc::now();
        
        let updated = diesel::update(
            notification_users::table
                .filter(notification_users::user_id.eq(user_id))
                .filter(notification_users::is_read.eq(false))
        )
        .set((
            notification_users::is_read.eq(true),
            notification_users::read_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(updated as i64)
    }

    #[instrument(skip(self, notification_ids))]
    async fn dismiss(&self, user_id: i32, notification_ids: Vec<i32>) -> Result<i64, ApplicationError> {
        debug!("🗑️ Descartando {} notificaciones", notification_ids.len());
        let mut conn = self.pool.get_connection().await?;
        let now = Utc::now();
        
        let updated = diesel::update(
            notification_users::table
                .filter(notification_users::user_id.eq(user_id))
                .filter(notification_users::notification_id.eq_any(&notification_ids))
        )
        .set((
            notification_users::is_dismissed.eq(true),
            notification_users::dismissed_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(updated as i64)
    }

    #[instrument(skip(self))]
    async fn dismiss_all(&self, user_id: i32) -> Result<i64, ApplicationError> {
        debug!("🗑️ Descartando todas las notificaciones para usuario: {}", user_id);
        let mut conn = self.pool.get_connection().await?;
        let now = Utc::now();
        
        let updated = diesel::update(
            notification_users::table
                .filter(notification_users::user_id.eq(user_id))
                .filter(notification_users::is_dismissed.eq(false))
        )
        .set((
            notification_users::is_dismissed.eq(true),
            notification_users::dismissed_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(updated as i64)
    }

    // ===== Utilidades =====

    #[instrument(skip(self, roles))]
    async fn get_users_by_roles(&self, roles: Vec<String>) -> Result<Vec<i32>, ApplicationError> {
        debug!("Buscando usuarios por roles: {:?}", roles);
        let mut conn = self.pool.get_connection().await?;
        
        let user_ids: Vec<i32> = users::table
            .filter(users::role.eq_any(&roles))
            .filter(users::is_active.eq(true))
            .select(users::id)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        debug!("Encontrados {} usuarios", user_ids.len());
        Ok(user_ids)
    }

    #[instrument(skip(self))]
    async fn get_all_active_user_ids(&self) -> Result<Vec<i32>, ApplicationError> {
        debug!("Obteniendo todos los usuarios activos");
        let mut conn = self.pool.get_connection().await?;
        
        let user_ids: Vec<i32> = users::table
            .filter(users::is_active.eq(true))
            .select(users::id)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(user_ids)
    }
}
