//! Tour Service - Lógica de negocio para tours
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con tours:
//! - Creación de tours
//! - Actualización de tours
//! - Desactivación (soft delete)
//! - Restauración
//! - Búsqueda
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateTourRequest, UpdateTourRequest, TourResponse,
};
use crate::application::ports::{TourRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Tour, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de tours - contiene la lógica de negocio
pub struct TourService {
    tour_repository: Arc<dyn TourRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl TourService {
    pub fn new(
        tour_repository: Arc<dyn TourRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            tour_repository,
            logging_service,
            notification_service,
        }
    }

    /// Listar tours con paginación
    #[instrument(skip(self))]
    pub async fn list_tours(
        &self,
        options: PaginationOptions,
        include_inactive: bool,
    ) -> Result<(Vec<TourResponse>, i64, i64), ApplicationError> {
        let result = if include_inactive {
            self.tour_repository.list_all_paginated(options).await?
        } else {
            self.tour_repository.list_paginated(options).await?
        };
        
        let total = result.total;
        let pages = result.pages();
        let current_page = result.current_page();
        let items: Vec<TourResponse> = result.data.into_iter().map(Into::into).collect();
        info!("📋 Listados {} tours (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener tour por ID
    #[instrument(skip(self))]
    pub async fn get_tour(&self, id: i32) -> Result<TourResponse, ApplicationError> {
        let tour = self.tour_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
        
        info!("🔍 Tour encontrado: {} (ID: {})", tour.nombre, id);
        Ok(TourResponse::from(tour))
    }

    /// Crear un nuevo tour
    #[instrument(skip(self, request))]
    pub async fn create_tour(
        &self,
        request: CreateTourRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<TourResponse, ApplicationError> {
        // Crear entidad de dominio
        let tour = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.tour_repository.create(&tour).await?;
        info!("✅ Tour creado: {} (ID: {})", created.nombre, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Tour>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Tour,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de creación de tour: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo tour creado",
            &format!("{} ha creado el tour '{}' ({} → {})", username, created.nombre, created.lugar_inicio, created.lugar_fin),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de tour creado: {}", e);
        }
        
        Ok(TourResponse::from(created))
    }

    /// Actualizar un tour existente
    #[instrument(skip(self, request))]
    pub async fn update_tour(
        &self,
        id: i32,
        request: UpdateTourRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<TourResponse, ApplicationError> {
        // Verificar que existe
        let old_tour = self.tour_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_tour, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_tour.clone(), Some(updated_by));
        
        // Persistir
        let result = self.tour_repository.update(&updated_entity).await?;
        info!("✏️ Tour actualizado: {} (ID: {})", result.nombre, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Tour>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Tour,
            id,
            Some(&old_tour),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("⚠️ Error al registrar log de actualización de tour: {}", e);
        }
        
        // Notificación si hubo cambios importantes
        if !changed_fields.is_empty() {
            let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
            if let Err(e) = self.notification_service.notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Tour actualizado",
                &format!("{} ha actualizado el tour '{}' (campos: {})", username, result.nombre, changed_fields.join(", ")),
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Low,
                Some(updated_by),
            ).await {
                warn!("⚠️ Error al enviar notificación de tour actualizado: {}", e);
            }
        }
        
        Ok(TourResponse::from(result))
    }

    /// Desactivar (soft delete) un tour
    #[instrument(skip(self))]
    pub async fn deactivate_tour(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener tour antes de desactivar
        let tour = self.tour_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
        
        // Desactivar
        let deleted = self.tour_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Tour {} no encontrado", id)));
        }
        
        info!("🗑️ Tour desactivado: {} (ID: {})", tour.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Tour>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Tour,
            id,
            Some(&tour),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de desactivación de tour: {}", e);
        }
        
        // Notificación a admins
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Tour desactivado",
            &format!("{} ha desactivado el tour '{}'", username, tour.nombre),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de tour desactivado: {}", e);
        }
        
        Ok(())
    }

    /// Restaurar un tour desactivado
    #[instrument(skip(self))]
    pub async fn restore_tour(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Restaurar
        let restored = self.tour_repository.restore(id, restored_by).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Tour {} no encontrado", id)));
        }
        
        info!("♻️ Tour restaurado: ID {}", id);
        
        // Obtener tour restaurado para el log
        let tour = self.tour_repository.find_by_id(id).await?.unwrap();
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Tour>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Tour,
            id,
            None,
            Some(&tour),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de restauración de tour: {}", e);
        }
        
        // Notificación a admins
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Tour restaurado",
            &format!("{} ha restaurado el tour '{}'", username, tour.nombre),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(restored_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de tour restaurado: {}", e);
        }
        
        Ok(())
    }

    /// Buscar tours por texto
    #[instrument(skip(self))]
    pub async fn search_tours(&self, query: &str) -> Result<Vec<TourResponse>, ApplicationError> {
        let tours = self.tour_repository
            .search(query)
            .await?;
        
        info!("🔍 Búsqueda '{}' encontró {} tours", query, tours.len());
        Ok(tours.into_iter().map(Into::into).collect())
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &Tour, request: &UpdateTourRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.nombre.as_ref().map(|n| n != &old.nombre).unwrap_or(false) {
            changed.push("nombre".to_string());
        }
        if request.lugar_inicio.as_ref().map(|l| l != &old.lugar_inicio).unwrap_or(false) {
            changed.push("lugar_inicio".to_string());
        }
        if request.lugar_fin.as_ref().map(|l| l != &old.lugar_fin).unwrap_or(false) {
            changed.push("lugar_fin".to_string());
        }
        if request.hora_inicio.as_ref().map(|h| Some(*h) != old.hora_inicio).unwrap_or(false) {
            changed.push("hora_inicio".to_string());
        }
        if request.hora_fin.as_ref().map(|h| Some(*h) != old.hora_fin).unwrap_or(false) {
            changed.push("hora_fin".to_string());
        }
        if request.duracion_dias.as_ref().map(|d| Some(*d) != old.duracion_dias).unwrap_or(false) {
            changed.push("duracion_dias".to_string());
        }
        if request.is_active.as_ref().map(|a| *a != old.is_active).unwrap_or(false) {
            changed.push("is_active".to_string());
        }
        
        changed
    }
}
