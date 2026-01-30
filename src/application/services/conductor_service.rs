//! Conductor Service - Lógica de negocio para conductores
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con conductores:
//! - CRUD de conductores
//! - Validación de brevete único
//! - Filtrado por transporte y disponibilidad
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateConductorRequest, UpdateConductorRequest, ConductorResponse, ConductorListItemDto,
};
use crate::application::ports::{ConductorRepositoryPort, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Conductor, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de conductores - contiene la lógica de negocio
pub struct ConductorService {
    conductor_repository: Arc<dyn ConductorRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl ConductorService {
    pub fn new(
        conductor_repository: Arc<dyn ConductorRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            conductor_repository,
            logging_service,
            notification_service,
        }
    }

    // ===== Métodos de consulta =====

    /// Listar conductores con detalles (nombre de persona y transporte)
    #[instrument(skip(self))]
    pub async fn list_conductores(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ConductorListItemDto>, i64), ApplicationError> {
        let (items, total) = self.conductor_repository.list_with_details(limit, offset).await?;
        info!("Listados {} conductores (offset: {}, total: {})", items.len(), offset, total);
        Ok((items, total))
    }

    /// Obtener conductor por ID
    #[instrument(skip(self))]
    pub async fn get_conductor(&self, id: i32) -> Result<ConductorResponse, ApplicationError> {
        let conductor = self.conductor_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
        
        info!("Conductor encontrado: {} (ID: {})", conductor.nro_brevete, id);
        Ok(ConductorResponse::from(conductor))
    }

    /// Listar conductores por transporte
    #[instrument(skip(self))]
    pub async fn list_by_transporte(&self, transporte_id: i32) -> Result<Vec<ConductorResponse>, ApplicationError> {
        let conductores = self.conductor_repository.find_by_transporte(transporte_id).await?;
        info!("🚗 Encontrados {} conductores para transporte {}", conductores.len(), transporte_id);
        Ok(conductores.into_iter().map(ConductorResponse::from).collect())
    }

    /// Listar conductores disponibles
    #[instrument(skip(self))]
    pub async fn list_available(&self) -> Result<Vec<ConductorResponse>, ApplicationError> {
        let conductores = self.conductor_repository.list_available().await?;
        info!("Encontrados {} conductores disponibles", conductores.len());
        Ok(conductores.into_iter().map(ConductorResponse::from).collect())
    }

    /// Listar conductores por transporte con paginación (para rol transportes)
    #[instrument(skip(self))]
    pub async fn list_conductores_by_transporte_paginated(
        &self,
        transporte_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ConductorListItemDto>, i64), ApplicationError> {
        let (items, total) = self.conductor_repository.list_by_transporte_paginated(transporte_id, limit, offset).await?;
        info!("Listados {} conductores del transporte {} (offset: {}, total: {})", items.len(), transporte_id, offset, total);
        Ok((items, total))
    }

    /// Listar conductores disponibles de un transporte específico
    #[instrument(skip(self))]
    pub async fn list_available_by_transporte(&self, transporte_id: i32) -> Result<Vec<ConductorResponse>, ApplicationError> {
        let conductores = self.conductor_repository.list_available_by_transporte(transporte_id).await?;
        info!("Encontrados {} conductores disponibles para transporte {}", conductores.len(), transporte_id);
        Ok(conductores.into_iter().map(ConductorResponse::from).collect())
    }

    // ===== Métodos de mutación =====

    /// Crear un nuevo conductor
    #[instrument(skip(self, request))]
    pub async fn create_conductor(
        &self,
        request: CreateConductorRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<ConductorResponse, ApplicationError> {
        // Validar brevete único
        if self.conductor_repository.exists_by_brevete(&request.nro_brevete).await? {
            return Err(ApplicationError::Conflict(format!("Brevete {} ya existe", request.nro_brevete)));
        }
        
        // Crear entidad de dominio
        let conductor = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.conductor_repository.create(&conductor).await?;
        info!("Conductor creado: {} (ID: {})", created.nro_brevete, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Conductor>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Conductor,
            created.id,
            &created.nro_brevete,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de conductor: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        let soat_status = if created.tiene_soat { "con SOAT" } else { "sin SOAT" };
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo conductor creado",
            &format!("{} ha creado el conductor con brevete '{}' ({})", username, created.nro_brevete, soat_status),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de conductor creado: {}", e);
        }
        
        Ok(ConductorResponse::from(created))
    }

    /// Actualizar un conductor existente
    #[instrument(skip(self, request))]
    pub async fn update_conductor(
        &self,
        id: i32,
        request: UpdateConductorRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<ConductorResponse, ApplicationError> {
        // Verificar que existe
        let old_conductor = self.conductor_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_conductor, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_conductor.clone(), Some(updated_by));
        
        // Persistir
        let result = self.conductor_repository.update(&updated_entity).await?;
        info!("✏️ Conductor actualizado: {} (ID: {})", result.nro_brevete, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Conductor>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Conductor,
            id,
            Some(&old_conductor),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de conductor: {}", e);
        }
        
        // Determinar prioridad - mayor si cambia tiene_soat
        let priority = if changed_fields.contains(&"tiene_soat".to_string()) {
            NotificationPriority::Normal
        } else {
            NotificationPriority::Low
        };
        
        // Notificación a admins
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        let fields_str = if changed_fields.is_empty() {
            "sin cambios específicos".to_string()
        } else {
            changed_fields.join(", ")
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Conductor actualizado",
            &format!("{} ha actualizado el conductor '{}'. Campos: {}", username, result.nro_brevete, fields_str),
            NotificationType::Info,
            NotificationCategory::Crud,
            priority,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de conductor actualizado: {}", e);
        }
        
        Ok(ConductorResponse::from(result))
    }

    /// Desactivar un conductor (soft delete)
    #[instrument(skip(self))]
    pub async fn delete_conductor(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let conductor = self.conductor_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
        
        // Soft delete
        if !self.conductor_repository.soft_delete(id, deleted_by).await? {
            return Err(ApplicationError::NotFound(format!("Conductor {} no encontrado", id)));
        }
        info!("[DELETE] Conductor desactivado: {} (ID: {})", conductor.nro_brevete, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Conductor>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Conductor,
            id,
            Some(&conductor),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de conductor: {}", e);
        }
        
        // Notificación a admins - Warning porque es una eliminación
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Conductor desactivado",
            &format!("{} ha desactivado el conductor con brevete '{}'", username, conductor.nro_brevete),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de conductor desactivado: {}", e);
        }
        
        Ok(())
    }

    /// Restaurar un conductor desactivado
    #[instrument(skip(self))]
    pub async fn restore_conductor(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Restore via repository
        if !self.conductor_repository.restore(id, restored_by).await? {
            return Err(ApplicationError::NotFound(format!("Conductor {} no encontrado", id)));
        }
        
        // Obtener conductor restaurado
        let conductor = self.conductor_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
        
        info!("♻️ Conductor restaurado: {} (ID: {})", conductor.nro_brevete, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Conductor>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Conductor,
            id,
            None,
            Some(&conductor),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de conductor: {}", e);
        }
        
        // Notificación a admins - Success porque se recupera
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Conductor restaurado",
            &format!("{} ha restaurado el conductor con brevete '{}'", username, conductor.nro_brevete),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de conductor restaurado: {}", e);
        }
        
        Ok(())
    }

    /// Eliminación permanente de conductor (hard delete) - Solo SuperAdmin
    #[instrument(skip(self))]
    pub async fn hard_delete_conductor(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener conductor antes de eliminar
        let conductor = self.conductor_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
        
        // Eliminar permanentemente
        if !self.conductor_repository.hard_delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Conductor {} no encontrado", id)));
        }
        info!("🗑️ Conductor ELIMINADO PERMANENTEMENTE: {} (ID: {})", conductor.nro_brevete, id);
        
        // Logging del evento (acción crítica)
        if let Err(e) = self.logging_service.log_delete::<Conductor>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Conductor,
            id,
            Some(&conductor),
            Some("HARD_DELETE - Eliminación permanente".to_string()),
        ).await {
            warn!("Error al registrar log de eliminación permanente de conductor: {}", e);
        }
        
        // Notificación CRÍTICA a SuperAdmin únicamente
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "⚠️ CONDUCTOR ELIMINADO PERMANENTEMENTE",
            &format!(
                "ACCIÓN CRÍTICA: {} ha eliminado PERMANENTEMENTE el conductor con brevete '{}'. Esta acción NO se puede deshacer.",
                username, conductor.nro_brevete
            ),
            NotificationType::Error,
            NotificationCategory::Crud,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de eliminación permanente de conductor: {}", e);
        }
        
        Ok(())
    }

    // ===== Métodos auxiliares privados =====

    /// Detectar campos que fueron modificados
    fn detect_changed_fields(&self, old: &Conductor, request: &UpdateConductorRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if let Some(id_persona) = request.id_persona {
            if id_persona != old.id_persona {
                changed.push("id_persona".to_string());
            }
        }
        if let Some(id_transporte) = request.id_transporte {
            if Some(id_transporte) != old.id_transporte {
                changed.push("id_transporte".to_string());
            }
        }
        if let Some(ref nro_brevete) = request.nro_brevete {
            if nro_brevete != &old.nro_brevete {
                changed.push("nro_brevete".to_string());
            }
        }
        if let Some(tiene_soat) = request.tiene_soat {
            if tiene_soat != old.tiene_soat {
                changed.push("tiene_soat".to_string());
            }
        }
        if request.status.is_some() {
            changed.push("status".to_string());
        }
        if let Some(is_active) = request.is_active {
            if is_active != old.is_active {
                changed.push("is_active".to_string());
            }
        }
        
        changed
    }
}
