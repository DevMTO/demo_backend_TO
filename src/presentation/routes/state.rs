//! Estado de la aplicación para las rutas

use std::sync::Arc;

use crate::infrastructure::container::DependencyContainer;
use crate::infrastructure::sse::NotificationBroadcaster;
use crate::application::dtos::UserNotificationDto;
use crate::domain::entities::{
    NotificationType, NotificationCategory, NotificationPriority, UserRole,
};
use crate::infrastructure::sse::SseEvent;
use crate::domain::errors::ApplicationError;

#[derive(Clone)]
pub struct AppState {
    pub container: Arc<DependencyContainer>,
    pub broadcaster: Arc<NotificationBroadcaster>,
}

impl AppState {
    /// Notificar a roles específicos y enviar por SSE
    pub async fn notify_roles_with_broadcast(
        &self,
        roles: Vec<UserRole>,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // 1. Crear notificación en DB
        let notification = self.container.notification_service.notify_roles(
            roles.clone(),
            title,
            message,
            notification_type.clone(),
            category.clone(),
            priority.clone(),
            created_by,
        ).await?;

        // 2. Obtener IDs de usuarios con esos roles
        let roles_str: Vec<String> = roles.iter().map(|r| r.to_string().to_lowercase()).collect();
        let user_ids = self.container.notification_repository.get_users_by_roles(roles_str).await?;

        // 3. Enviar por SSE a cada usuario conectado
        let dto = UserNotificationDto {
            id: notification.id,
            title: notification.title,
            message: notification.message,
            notification_type: notification.notification_type.to_string(),
            category: notification.category.to_string(),
            priority: notification.priority.to_string(),
            entity_type: notification.entity_type.clone(),
            entity_id: notification.entity_id,
            metadata: notification.metadata.clone(),
            is_read: false,
            read_at: None,
            is_dismissed: false,
            created_at: notification.created_at,
        };

        let event = SseEvent::NewNotification(dto);
        for user_id in user_ids {
            // Excluir al usuario que creó la notificación del broadcast SSE
            if Some(user_id) != created_by {
                self.broadcaster.send_to_user(user_id, event.clone()).await;
            }
        }

        Ok(())
    }
}
