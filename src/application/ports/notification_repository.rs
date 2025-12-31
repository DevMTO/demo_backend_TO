use async_trait::async_trait;
use crate::domain::entities::{
    Notification, NewNotification,
    NotificationUser,
    NotificationWithReadStatus,
};
use crate::domain::errors::ApplicationError;
use super::generic_repository::{PaginationOptions, PaginatedResult};

#[derive(Debug, Clone, Default)]
pub struct NotificationFilters {
    pub notification_type: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub is_read: Option<bool>,
    pub is_dismissed: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PriorityCount {
    pub priority: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default)]
pub struct CleanupResult {
    pub deleted_expired: i64,
    pub deleted_low: i64,
    pub deleted_normal: i64,
    pub deleted_high: i64,
    pub deleted_urgent: i64,
    pub total_deleted: i64,
}

#[async_trait]
pub trait NotificationRepositoryPort: Send + Sync {
    // ===== Notificaciones =====
    
    /// Crear nueva notificación
    async fn create(&self, notification: NewNotification) -> Result<Notification, ApplicationError>;
    
    /// Obtener notificación por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<Notification>, ApplicationError>;
    
    /// Listar notificaciones (admin view)
    async fn find_all(
        &self,
        filters: NotificationFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>, ApplicationError>;
    
    /// Contar notificaciones totales
    async fn count(&self, filters: NotificationFilters) -> Result<i64, ApplicationError>;
    
    /// Eliminar notificación
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    
    /// Eliminar notificaciones expiradas
    #[allow(dead_code)]
    async fn cleanup_expired(&self) -> Result<i64, ApplicationError>;
    
    /// Cleanup completo de notificaciones (usando función de DB)
    /// Retorna: (expired, low, normal, high, urgent, total)
    async fn cleanup_by_priority(
        &self,
        days_low: i32,
        days_normal: i32,
        days_high: i32,
        days_urgent: i32,
    ) -> Result<CleanupResult, ApplicationError>;
    
    // ===== Notificaciones de Usuario =====
    
    /// Crear relación usuario-notificación
    async fn create_user_notification(
        &self, 
        notification_id: i32, 
        user_id: i32
    ) -> Result<NotificationUser, ApplicationError>;
    
    /// Crear relaciones para múltiples usuarios
    async fn create_user_notifications_batch(
        &self,
        notification_id: i32,
        user_ids: Vec<i32>
    ) -> Result<Vec<NotificationUser>, ApplicationError>;
    
    /// Obtener notificaciones de un usuario con estado de lectura
    async fn find_user_notifications(
        &self,
        user_id: i32,
        filters: NotificationFilters,
        pagination: PaginationOptions
    ) -> Result<PaginatedResult<NotificationWithReadStatus>, ApplicationError>;
    
    /// Contar notificaciones de un usuario
    async fn count_user_notifications(
        &self,
        user_id: i32,
        unread_only: bool,
    ) -> Result<i64, ApplicationError>;
    
    /// Contar notificaciones no leídas de un usuario
    async fn count_unread(&self, user_id: i32) -> Result<i64, ApplicationError>;
    
    /// Contar no leídas por prioridad
    #[allow(dead_code)]
    async fn count_unread_by_priority(&self, user_id: i32) -> Result<Vec<PriorityCount>, ApplicationError>;
    
    /// Marcar notificación(es) como leída(s)
    async fn mark_as_read(&self, user_id: i32, notification_ids: Vec<i32>) -> Result<i64, ApplicationError>;
    
    /// Marcar todas como leídas
    async fn mark_all_as_read(&self, user_id: i32) -> Result<i64, ApplicationError>;
    
    /// Descartar notificación(es)
    async fn dismiss(&self, user_id: i32, notification_ids: Vec<i32>) -> Result<i64, ApplicationError>;
    
    /// Descartar todas las notificaciones
    async fn dismiss_all(&self, user_id: i32) -> Result<i64, ApplicationError>;
    
    // ===== Utilidades =====
    
    /// Obtener usuarios por rol (para envío de notificaciones por rol)
    async fn get_users_by_roles(&self, roles: Vec<String>) -> Result<Vec<i32>, ApplicationError>;
    
    /// Obtener todos los user_ids activos
    async fn get_all_active_user_ids(&self) -> Result<Vec<i32>, ApplicationError>;
}
