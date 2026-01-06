use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;

use crate::application::ports::NotificationServicePort;
use crate::application::services::NotificationService;
use crate::application::dtos::UserNotificationDto;
use crate::domain::entities::{
    UserRole, NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::sse::{NotificationBroadcaster, SseEvent};

/// Adaptador que combina el servicio de notificaciones con SSE
pub struct NotificationBroadcastAdapter {
    notification_service: Arc<NotificationService>,
    notification_repository: Arc<dyn crate::application::ports::NotificationRepositoryPort>,
    broadcaster: Arc<NotificationBroadcaster>,
}

impl NotificationBroadcastAdapter {
    pub fn new(
        notification_service: Arc<NotificationService>,
        notification_repository: Arc<dyn crate::application::ports::NotificationRepositoryPort>,
        broadcaster: Arc<NotificationBroadcaster>,
    ) -> Self {
        Self {
            notification_service,
            notification_repository,
            broadcaster,
        }
    }
}

#[async_trait]
impl NotificationServicePort for NotificationBroadcastAdapter {
    async fn notify_roles(
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
        let notification = self.notification_service.notify_roles(
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
        let user_ids = self.notification_repository.get_users_by_roles(roles_str).await?;

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

    async fn notify_user(
        &self,
        user_id: i32,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // 1. Crear notificación en DB
        let notification = self.notification_service.notify_user(
            user_id,
            title,
            message,
            notification_type.clone(),
            category.clone(),
            priority.clone(),
            created_by,
        ).await?;

        // 2. Enviar por SSE al usuario
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
        self.broadcaster.send_to_user(user_id, event).await;

        Ok(())
    }

    async fn notify_all(
        &self,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // 1. Crear notificación en DB para todos
        let notification = self.notification_service.notify_all(
            title,
            message,
            notification_type.clone(),
            category.clone(),
            priority.clone(),
            created_by,
        ).await?;

        // 2. Enviar por SSE a todos los usuarios conectados
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
        
        // Obtener todos los usuarios activos y enviarles
        match self.notification_repository.get_all_active_user_ids().await {
            Ok(user_ids) => {
                for user_id in user_ids {
                    if Some(user_id) != created_by {
                        self.broadcaster.send_to_user(user_id, event.clone()).await;
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Error al obtener usuarios para broadcast: {}", e);
            }
        }

        Ok(())
    }
}
