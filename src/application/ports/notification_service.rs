use async_trait::async_trait;
use crate::domain::entities::{
    UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Port para el servicio de notificaciones
/// 
/// Este trait permite desacoplar la lógica de negocio del mecanismo
/// específico de envío de notificaciones (base de datos, SSE, etc.)
#[async_trait]
pub trait NotificationServicePort: Send + Sync {
    /// Notificar a roles específicos
    /// 
    /// Crea una notificación en la base de datos y opcionalmente
    /// la envía por SSE a los usuarios conectados.
    async fn notify_roles(
        &self,
        roles: Vec<UserRole>,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError>;
    
    /// Notificar a un usuario específico
    /// 
    /// Crea una notificación para un usuario y opcionalmente
    /// la envía por SSE si está conectado.
    async fn notify_user(
        &self,
        user_id: i32,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError>;
    
    /// Notificar a todos los usuarios activos
    #[allow(dead_code)]
    async fn notify_all(
        &self,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError>;
    
    /// Notificar a roles específicos de una entidad
    /// 
    /// Crea una notificación solo para usuarios que tengan el rol especificado
    /// Y que pertenezcan a la entidad indicada (filtrado por id_entidad).
    /// Útil para notificar solo a los contadores de una agencia específica.
    async fn notify_roles_for_entity(
        &self,
        roles: Vec<UserRole>,
        entity_id: i32,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError>;
}
