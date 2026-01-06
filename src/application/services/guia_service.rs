use std::sync::Arc;
use tracing::info;

use crate::application::dtos::GuiaListItemDto;
use crate::application::ports::{GuiaRepositoryPort, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Guia, EntityType, NotificationCategory, NotificationPriority, NotificationType, UserRole,
};
use crate::domain::errors::ApplicationError;

/// GuiaService - Service layer for Guia (tour guide) business logic
/// 
/// Following hexagonal architecture, this service encapsulates:
/// - Business logic for guia CRUD operations
/// - Activity logging for all operations
/// - Real-time notifications via SSE broadcast
pub struct GuiaService {
    guia_repository: Arc<dyn GuiaRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl GuiaService {
    pub fn new(
        guia_repository: Arc<dyn GuiaRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            guia_repository,
            logging_service,
            notification_service,
        }
    }

    // ==================== READ OPERATIONS ====================

    /// List all guias with pagination (includes persona info)
    pub async fn list_guias_with_persona(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<GuiaListItemDto>, i64), ApplicationError> {
        self.guia_repository.list_with_persona(limit, offset).await
    }

    /// Get a specific guia by ID
    pub async fn get_guia(&self, id: i32) -> Result<Guia, ApplicationError> {
        self.guia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))
    }

    /// Search guias by idioma (language)
    pub async fn search_by_idioma(&self, idioma: &str) -> Result<Vec<Guia>, ApplicationError> {
        self.guia_repository.find_by_idioma(idioma).await
    }

    /// Search guias by especialidad (specialty)
    pub async fn search_by_especialidad(&self, especialidad: &str) -> Result<Vec<Guia>, ApplicationError> {
        self.guia_repository.find_by_especialidad(especialidad).await
    }

    /// List all available guias (active)
    pub async fn list_available(&self) -> Result<Vec<Guia>, ApplicationError> {
        self.guia_repository.list_available().await
    }

    // ==================== WRITE OPERATIONS ====================

    /// Create a new guia with logging and notifications
    pub async fn create_guia(
        &self,
        guia: &Guia,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<Guia, ApplicationError> {
        // Create the guia
        let created = self.guia_repository.create(guia).await?;
        info!("✅ Guía creado: {} (ID: {})", created.nro_carnet, created.id);

        // Log activity
        let _ = self
            .logging_service
            .log_create::<Guia>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Guia,
                created.id,
                &created.nro_carnet,
                Some(&created),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Nuevo guía creado",
                &format!(
                    "{} ha creado el guía con carnet '{}'",
                    actor_username, created.nro_carnet
                ),
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Low,
                Some(actor_id),
            )
            .await;

        Ok(created)
    }

    /// Update an existing guia with logging and notifications
    pub async fn update_guia(
        &self,
        id: i32,
        updated_guia: &Guia,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<Guia, ApplicationError> {
        // Get old version for comparison
        let old_guia = self
            .guia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))?;

        // Update the guia
        let result = self.guia_repository.update(updated_guia).await?;
        info!("📝 Guía actualizado: {} (ID: {})", result.nro_carnet, result.id);

        // Detect changed fields for logging
        let changed_fields = self.detect_changed_fields(&old_guia, &result);

        // Log activity
        let _ = self
            .logging_service
            .log_update::<Guia>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Guia,
                result.id,
                Some(&old_guia),
                Some(&result),
                Some(changed_fields.clone()),
                None,
            )
            .await;

        // Determine notification priority based on changes
        let priority = if changed_fields.contains(&"idiomas".to_string())
            || changed_fields.contains(&"especialidades".to_string())
        {
            NotificationPriority::Normal
        } else {
            NotificationPriority::Low
        };

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Guía actualizado",
                &format!(
                    "{} ha actualizado el guía con carnet '{}'",
                    actor_username, result.nro_carnet
                ),
                NotificationType::Info,
                NotificationCategory::Crud,
                priority,
                Some(actor_id),
            )
            .await;

        Ok(result)
    }

    /// Hard delete a guia with logging and notifications
    pub async fn delete_guia(
        &self,
        id: i32,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<(), ApplicationError> {
        // Get guia info before deleting
        let guia = self
            .guia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))?;

        // Delete
        if !self.guia_repository.delete(id).await? {
            return Err(ApplicationError::NotFound(format!(
                "Guía {} no encontrado",
                id
            )));
        }
        info!("🗑️ Guía eliminado: {} (ID: {})", guia.nro_carnet, id);

        // Log activity
        let _ = self
            .logging_service
            .log_delete::<Guia>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Guia,
                id,
                Some(&guia),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Guía eliminado",
                &format!(
                    "{} ha eliminado el guía con carnet '{}'",
                    actor_username, guia.nro_carnet
                ),
                NotificationType::Warning,
                NotificationCategory::Crud,
                NotificationPriority::Normal,
                Some(actor_id),
            )
            .await;

        Ok(())
    }

    // ==================== PRIVATE HELPERS ====================

    /// Detect which fields changed between old and new guia
    fn detect_changed_fields(&self, old: &Guia, new: &Guia) -> Vec<String> {
        let mut changed = Vec::new();

        if old.id_persona != new.id_persona {
            changed.push("id_persona".to_string());
        }
        if old.nro_carnet != new.nro_carnet {
            changed.push("nro_carnet".to_string());
        }
        if old.idiomas != new.idiomas {
            changed.push("idiomas".to_string());
        }
        if old.especialidades != new.especialidades {
            changed.push("especialidades".to_string());
        }
        if old.status != new.status {
            changed.push("status".to_string());
        }
        if old.is_active != new.is_active {
            changed.push("is_active".to_string());
        }

        changed
    }
}
