use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::application::ports::{NotificationRepositoryPort, NotificationFilters as PortFilters, PaginationOptions};
use crate::application::dtos::CreateNotificationRequest;
use crate::domain::entities::{
    NotificationBuilder, NotificationType, NotificationCategory, NotificationPriority,
    EntityType,
};
use crate::domain::errors::ApplicationError;

/// Servicio de notificaciones
pub struct NotificationService {
    repository: Arc<dyn NotificationRepositoryPort>,
}

#[allow(dead_code)]
impl NotificationService {
    pub fn new(repository: Arc<dyn NotificationRepositoryPort>) -> Self {
        Self { repository }
    }

    /// Crear notificación desde request
    #[instrument(skip(self, request))]
    pub async fn create_notification(
        &self,
        request: CreateNotificationRequest,
        created_by: Option<i32>,
    ) -> Result<crate::domain::entities::Notification, ApplicationError> {
        let mut builder = NotificationBuilder::new(&request.title, &request.message)
            .notification_type(NotificationType::from(request.notification_type.as_str()))
            .category(NotificationCategory::from(request.category.as_str()))
            .priority(NotificationPriority::from(request.priority.as_str()));

        if let Some(entity_type) = &request.entity_type {
            if let Some(entity_id) = request.entity_id {
                builder = builder.entity(entity_type, entity_id);
            }
        }
        if let Some(metadata) = request.metadata.clone() {
            builder = builder.metadata(metadata);
        }
        if let Some(roles) = request.target_roles.clone() {
            builder = builder.for_roles(roles);
        }
        if let Some(user_id) = request.target_user_id {
            builder = builder.for_user(user_id);
        }
        if let Some(expires) = request.expires_at {
            builder = builder.expires_at(expires);
        }
        if let Some(uid) = created_by {
            builder = builder.created_by(uid);
        }

        let new_notification = builder.build();
        let notification = self.repository.create(new_notification).await?;
        
        // Distribuir a usuarios
        self.distribute_notification(&notification.id, request.target_user_id, request.target_roles).await?;

        Ok(notification)
    }

    /// Notificación de sistema para todos los usuarios
    #[instrument(skip(self))]
    pub async fn notify_all(
        &self,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<crate::domain::entities::Notification, ApplicationError> {
        let mut builder = NotificationBuilder::new(title, message)
            .notification_type(notification_type)
            .category(category)
            .priority(priority);

        if let Some(uid) = created_by {
            builder = builder.created_by(uid);
        }

        let notification = self.repository.create(builder.build()).await?;
        
        // Distribuir a todos los usuarios activos
        let user_ids = self.repository.get_all_active_user_ids().await?;
        if !user_ids.is_empty() {
            self.repository.create_user_notifications_batch(notification.id, user_ids).await?;
        }

        info!("📢 Notificación enviada a todos los usuarios: {}", title);
        Ok(notification)
    }

    /// Notificación para roles específicos
    #[instrument(skip(self, roles))]
    pub async fn notify_roles(
        &self,
        roles: Vec<crate::domain::entities::UserRole>,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<crate::domain::entities::Notification, ApplicationError> {
        let roles_str: Vec<String> = roles.iter().map(|r| r.to_string().to_lowercase()).collect();
        
        let mut builder = NotificationBuilder::new(title, message)
            .notification_type(notification_type)
            .category(category)
            .priority(priority)
            .for_roles(roles_str.clone());

        if let Some(uid) = created_by {
            builder = builder.created_by(uid);
        }

        let notification = self.repository.create(builder.build()).await?;
        
        // Distribuir a usuarios con esos roles
        let user_ids = self.repository.get_users_by_roles(roles_str).await?;
        if !user_ids.is_empty() {
            self.repository.create_user_notifications_batch(notification.id, user_ids).await?;
        }

        Ok(notification)
    }

    /// Notificación para un usuario específico
    #[instrument(skip(self))]
    pub async fn notify_user(
        &self,
        user_id: i32,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<crate::domain::entities::Notification, ApplicationError> {
        let mut builder = NotificationBuilder::new(title, message)
            .notification_type(notification_type)
            .category(category)
            .priority(priority)
            .for_user(user_id);

        if let Some(uid) = created_by {
            builder = builder.created_by(uid);
        }

        let notification = self.repository.create(builder.build()).await?;
        
        // Crear relación directa con el usuario
        self.repository.create_user_notification(notification.id, user_id).await?;

        debug!("📬 Notificación enviada al usuario {}: {}", user_id, title);
        Ok(notification)
    }

    // ===== Métodos de alto nivel para eventos del sistema =====

    /// Notificación de creación de entidad
    #[instrument(skip(self))]
    pub async fn notify_entity_created(
        &self,
        entity_type: EntityType,
        entity_id: i32,
        entity_name: &str,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        let title = format!("Nuevo {} creado", entity_type.as_str());
        let message = format!("Se ha creado un nuevo {}: {}", entity_type.as_str(), entity_name);

        // Notificar a admins
        self.notify_roles(
            vec![crate::domain::entities::UserRole::SuperAdmin, crate::domain::entities::UserRole::Admin],
            &title,
            &message,
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            created_by,
        ).await?;

        Ok(())
    }

    /// Notificación de actualización de entidad
    #[instrument(skip(self))]
    pub async fn notify_entity_updated(
        &self,
        entity_type: EntityType,
        entity_id: i32,
        entity_name: &str,
        updated_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        let title = format!("{} actualizado", entity_type.as_str());
        let message = format!("Se ha actualizado {}: {}", entity_type.as_str(), entity_name);

        // Notificar a admins
        self.notify_roles(
            vec![crate::domain::entities::UserRole::SuperAdmin, crate::domain::entities::UserRole::Admin],
            &title,
            &message,
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            updated_by,
        ).await?;

        Ok(())
    }

    /// Notificación de eliminación de entidad
    #[instrument(skip(self))]
    pub async fn notify_entity_deleted(
        &self,
        entity_type: EntityType,
        entity_id: i32,
        entity_name: &str,
        deleted_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        let title = format!("{} eliminado", entity_type.as_str());
        let message = format!("Se ha eliminado {}: {}", entity_type.as_str(), entity_name);

        // Notificar a admins
        self.notify_roles(
            vec![crate::domain::entities::UserRole::SuperAdmin, crate::domain::entities::UserRole::Admin],
            &title,
            &message,
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            deleted_by,
        ).await?;

        Ok(())
    }

    /// Notificación de login
    #[instrument(skip(self))]
    pub async fn notify_login(
        &self,
        user_id: i32,
        username: &str,
        ip: Option<&str>,
    ) -> Result<(), ApplicationError> {
        let message = match ip {
            Some(ip) => format!("Inicio de sesión desde IP: {}", ip),
            None => "Inicio de sesión detectado".to_string(),
        };

        self.notify_user(
            user_id,
            "Nuevo inicio de sesión",
            &message,
            NotificationType::Info,
            NotificationCategory::Auth,
            NotificationPriority::Low,
            None,
        ).await?;

        Ok(())
    }

    // ===== Métodos de consulta =====

    /// Obtener notificaciones de un usuario
    #[instrument(skip(self))]
    pub async fn get_user_notifications(
        &self,
        user_id: i32,
        unread_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::domain::entities::NotificationWithReadStatus>, ApplicationError> {
        let port_filters = PortFilters {
            notification_type: None,
            category: None,
            priority: None,
            is_read: if unread_only { Some(false) } else { None },
            is_dismissed: Some(false), // No mostrar descartadas
        };

        let pagination = PaginationOptions {
            limit: Some(limit),
            offset: Some(offset),
        };

        let result = self.repository.find_user_notifications(user_id, port_filters, pagination).await?;
        Ok(result.data)
    }

    /// Obtener contador de no leídas
    #[instrument(skip(self))]
    pub async fn get_unread_count(&self, user_id: i32) -> Result<i64, ApplicationError> {
        self.repository.count_unread(user_id).await
    }

    /// Marcar una notificación como leída
    #[instrument(skip(self))]
    pub async fn mark_as_read(
        &self,
        user_id: i32,
        notification_id: i32,
    ) -> Result<(), ApplicationError> {
        self.repository.mark_as_read(user_id, vec![notification_id]).await?;
        Ok(())
    }

    /// Marcar todas como leídas
    #[instrument(skip(self))]
    pub async fn mark_all_as_read(&self, user_id: i32) -> Result<i64, ApplicationError> {
        self.repository.mark_all_as_read(user_id).await
    }

    /// Descartar una notificación
    #[instrument(skip(self))]
    pub async fn dismiss(
        &self,
        user_id: i32,
        notification_id: i32,
    ) -> Result<(), ApplicationError> {
        self.repository.dismiss(user_id, vec![notification_id]).await?;
        Ok(())
    }

    /// Descartar todas
    #[instrument(skip(self))]
    pub async fn dismiss_all(&self, user_id: i32) -> Result<i64, ApplicationError> {
        self.repository.dismiss_all(user_id).await
    }

    // ===== Helpers privados =====

    /// Distribuir notificación a usuarios
    async fn distribute_notification(
        &self,
        notification_id: &i32,
        target_user_id: Option<i32>,
        target_roles: Option<Vec<String>>,
    ) -> Result<(), ApplicationError> {
        if let Some(user_id) = target_user_id {
            // Notificación para usuario específico
            self.repository.create_user_notification(*notification_id, user_id).await?;
        } else if let Some(roles) = target_roles {
            // Notificación para roles
            let user_ids = self.repository.get_users_by_roles(roles).await?;
            if !user_ids.is_empty() {
                self.repository.create_user_notifications_batch(*notification_id, user_ids).await?;
            }
        } else {
            // Notificación global - distribuir a todos
            let user_ids = self.repository.get_all_active_user_ids().await?;
            if !user_ids.is_empty() {
                self.repository.create_user_notifications_batch(*notification_id, user_ids).await?;
            }
        }

        Ok(())
    }
}
