use std::sync::Arc;
use tracing::info;

use crate::application::ports::{EntradaRepositoryPort, NotificationServicePort, PaginatedResult, PaginationOptions};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Entrada, EntityType, NotificationCategory, NotificationPriority, NotificationType, UserRole,
};
use crate::domain::errors::ApplicationError;

/// EntradaService - Service layer for Entrada (entry/ticket) business logic
/// 
/// Following hexagonal architecture, this service encapsulates:
/// - Business logic for entrada CRUD operations
/// - Activity logging for all operations
/// - Real-time notifications via SSE broadcast
pub struct EntradaService {
    entrada_repository: Arc<dyn EntradaRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl EntradaService {
    pub fn new(
        entrada_repository: Arc<dyn EntradaRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            entrada_repository,
            logging_service,
            notification_service,
        }
    }

    // ==================== READ OPERATIONS ====================

    /// List all active entradas with pagination
    pub async fn list_entradas(
        &self,
        options: PaginationOptions,
    ) -> Result<PaginatedResult<Entrada>, ApplicationError> {
        self.entrada_repository.list_paginated(options).await
    }
    
    /// List ALL entradas (active + inactive) with pagination
    pub async fn list_all_entradas(
        &self,
        options: PaginationOptions,
    ) -> Result<PaginatedResult<Entrada>, ApplicationError> {
        self.entrada_repository.list_all_paginated(options).await
    }

    /// Get a specific entrada by ID
    pub async fn get_entrada(&self, id: i32) -> Result<Entrada, ApplicationError> {
        self.entrada_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))
    }

    // ==================== WRITE OPERATIONS ====================

    /// Create a new entrada with logging and notifications
    pub async fn create_entrada(
        &self,
        entrada: &Entrada,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<Entrada, ApplicationError> {
        // Create the entrada
        let created = self.entrada_repository.create(entrada).await?;
        info!("Entrada creada: {} (ID: {})", created.nombre, created.id);

        // Log activity
        let _ = self
            .logging_service
            .log_create::<Entrada>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Entrada,
                created.id,
                &created.nombre,
                Some(&created),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Nueva entrada creada",
                &format!(
                    "{} ha creado la entrada '{}'",
                    actor_username, created.nombre
                ),
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Low,
                Some(actor_id),
            )
            .await;

        Ok(created)
    }

    /// Update an existing entrada with logging and notifications
    pub async fn update_entrada(
        &self,
        id: i32,
        updated_entrada: &Entrada,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<Entrada, ApplicationError> {
        // Get old version for comparison
        let old_entrada = self
            .entrada_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;

        // Update the entrada
        let result = self.entrada_repository.update(updated_entrada).await?;
        info!("Entrada actualizada: {} (ID: {})", result.nombre, result.id);

        // Detect changed fields for logging
        let changed_fields = self.detect_changed_fields(&old_entrada, &result);

        // Log activity
        let _ = self
            .logging_service
            .log_update::<Entrada>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Entrada,
                result.id,
                Some(&old_entrada),
                Some(&result),
                Some(changed_fields.clone()),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Entrada actualizada",
                &format!(
                    "{} ha actualizado la entrada '{}'",
                    actor_username, result.nombre
                ),
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Low,
                Some(actor_id),
            )
            .await;

        Ok(result)
    }

    /// Soft delete (deactivate) an entrada with logging and notifications
    pub async fn deactivate_entrada(
        &self,
        id: i32,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<(), ApplicationError> {
        // Get entrada info before deleting
        let entrada = self
            .entrada_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;

        // Soft delete
        if !self.entrada_repository.soft_delete(id, actor_id).await? {
            return Err(ApplicationError::NotFound(format!(
                "Entrada {} no encontrada",
                id
            )));
        }
        info!("🗑️ Entrada desactivada: {} (ID: {})", entrada.nombre, id);

        // Log activity
        let _ = self
            .logging_service
            .log_delete::<Entrada>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Entrada,
                id,
                Some(&entrada),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Entrada eliminada",
                &format!(
                    "{} ha eliminado la entrada '{}'",
                    actor_username, entrada.nombre
                ),
                NotificationType::Warning,
                NotificationCategory::Crud,
                NotificationPriority::Normal,
                Some(actor_id),
            )
            .await;

        Ok(())
    }

    /// Restore a deactivated entrada with logging and notifications
    pub async fn restore_entrada(
        &self,
        id: i32,
        actor_id: i32,
        actor_username: &str,
    ) -> Result<Entrada, ApplicationError> {
        // Restore the entrada
        if !self.entrada_repository.restore(id, actor_id).await? {
            return Err(ApplicationError::NotFound(format!(
                "Entrada {} no encontrada",
                id
            )));
        }

        // Get entrada info after restore
        let entrada = self
            .entrada_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;

        info!("♻️ Entrada restaurada: {} (ID: {})", entrada.nombre, id);

        // Log activity
        let _ = self
            .logging_service
            .log_update::<Entrada>(
                Some(actor_id),
                Some(actor_username.to_string()),
                EntityType::Entrada,
                id,
                None,
                Some(&entrada),
                Some(vec!["is_active".to_string()]),
                None,
            )
            .await;

        // Notify admins via SSE broadcast
        let _ = self
            .notification_service
            .notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Entrada restaurada",
                &format!(
                    "{} ha restaurado la entrada '{}'",
                    actor_username, entrada.nombre
                ),
                NotificationType::Success,
                NotificationCategory::Crud,
                NotificationPriority::Low,
                Some(actor_id),
            )
            .await;

        Ok(entrada)
    }

    // ==================== PRIVATE HELPERS ====================

    /// Detect which fields changed between old and new entrada
    fn detect_changed_fields(&self, old: &Entrada, new: &Entrada) -> Vec<String> {
        let mut changed = Vec::new();

        if old.nombre != new.nombre {
            changed.push("nombre".to_string());
        }
        if old.descripcion != new.descripcion {
            changed.push("descripcion".to_string());
        }
        if old.tours_asociados != new.tours_asociados {
            changed.push("tours_asociados".to_string());
        }
        if old.is_active != new.is_active {
            changed.push("is_active".to_string());
        }

        changed
    }
}

