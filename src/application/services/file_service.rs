//! File Service - Lógica de negocio para archivos de viaje (Files)
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con files:
//! - Creación de files
//! - Actualización de files
//! - Búsqueda
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use chrono::NaiveDate;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateFileRequest, UpdateFileRequest, FileResponse,
};
use crate::application::ports::{FileRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    File, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de files - contiene la lógica de negocio
pub struct FileService {
    file_repository: Arc<dyn FileRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl FileService {
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            file_repository,
            logging_service,
            notification_service,
        }
    }

    /// Listar files con paginación
    #[instrument(skip(self))]
    pub async fn list_files(
        &self,
        options: PaginationOptions,
    ) -> Result<(Vec<FileResponse>, i64, i64), ApplicationError> {
        let result = self.file_repository
            .list_paginated(options)
            .await?;
        
        let total = result.total;
        let pages = result.pages();
        let current_page = result.current_page();
        let items: Vec<FileResponse> = result.data.into_iter().map(Into::into).collect();
        info!("📋 Listados {} files (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener file por ID
    #[instrument(skip(self))]
    pub async fn get_file(&self, id: i32) -> Result<FileResponse, ApplicationError> {
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        info!("🔍 File encontrado: ID {}", id);
        Ok(FileResponse::from(file))
    }

    /// Crear un nuevo file
    #[instrument(skip(self, request))]
    pub async fn create_file(
        &self,
        request: CreateFileRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<FileResponse, ApplicationError> {
        // Crear entidad de dominio
        let file = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.file_repository.create(&file).await?;
        info!("✅ File creado: ID {} para fechas {} - {}", created.id, created.fecha_inicio, created.fecha_fin);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<File>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::File,
            created.id,
            &format!("File #{} - {} a {}", created.id, created.fecha_inicio, created.fecha_fin),
            Some(&created),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de creación de file: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo file creado",
            &format!("{} ha creado el file #{} para {} - {}", username, created.id, created.fecha_inicio, created.fecha_fin),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(created_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de file creado: {}", e);
        }
        
        Ok(FileResponse::from(created))
    }

    /// Actualizar un file existente
    #[instrument(skip(self, request))]
    pub async fn update_file(
        &self,
        id: i32,
        request: UpdateFileRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<FileResponse, ApplicationError> {
        // Verificar que existe
        let old_file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_file, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_file.clone(), Some(updated_by));
        
        // Persistir
        let result = self.file_repository.update(&updated_entity).await?;
        info!("✏️ File actualizado: ID {}", result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::File,
            id,
            Some(&old_file),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("⚠️ Error al registrar log de actualización de file: {}", e);
        }
        
        // Notificación si hubo cambios importantes (especialmente estado)
        if !changed_fields.is_empty() {
            let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
            let priority = if changed_fields.contains(&"status".to_string()) {
                NotificationPriority::High
            } else {
                NotificationPriority::Normal
            };
            
            if let Err(e) = self.notification_service.notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "File actualizado",
                &format!("{} ha actualizado file #{} (campos: {})", username, id, changed_fields.join(", ")),
                NotificationType::Info,
                NotificationCategory::Crud,
                priority,
                Some(updated_by),
            ).await {
                warn!("⚠️ Error al enviar notificación de file actualizado: {}", e);
            }
        }
        
        Ok(FileResponse::from(result))
    }

    /// Eliminar un file
    #[instrument(skip(self))]
    pub async fn delete_file(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener file antes de eliminar
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Eliminar
        let deleted = self.file_repository.delete(id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        info!("🗑️ File eliminado: ID {}", id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<File>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::File,
            id,
            Some(&file),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de eliminación de file: {}", e);
        }
        
        // Notificación a admins (eliminar es crítico)
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File eliminado",
            &format!("{} ha eliminado el file #{} (fechas: {} - {})", username, id, file.fecha_inicio, file.fecha_fin),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(deleted_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de file eliminado: {}", e);
        }
        
        Ok(())
    }

    /// Listar files por agencia
    #[instrument(skip(self))]
    pub async fn list_files_by_agencia(&self, agencia_id: i32) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_by_agencia(agencia_id)
            .await?;
        
        info!("📋 {} files encontrados para agencia {}", files.len(), agencia_id);
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Buscar files por rango de fechas
    #[instrument(skip(self))]
    pub async fn search_files_by_date_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_by_date_range(from, to)
            .await?;
        
        info!("📋 {} files encontrados entre {} y {}", files.len(), from, to);
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Listar files próximos (en los próximos 7 días)
    #[instrument(skip(self))]
    pub async fn list_files_upcoming(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_upcoming()
            .await?;
        
        info!("📋 {} files próximos encontrados", files.len());
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Listar files con pago pendiente
    #[instrument(skip(self))]
    pub async fn list_files_pending_payment(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_pending_payment()
            .await?;
        
        info!("📋 {} files con pago pendiente encontrados", files.len());
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &File, request: &UpdateFileRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.fecha_inicio.as_ref().map(|f| *f != old.fecha_inicio).unwrap_or(false) {
            changed.push("fecha_inicio".to_string());
        }
        if request.fecha_fin.as_ref().map(|f| *f != old.fecha_fin).unwrap_or(false) {
            changed.push("fecha_fin".to_string());
        }
        if request.id_tour.as_ref().map(|t| *t != old.id_tour).unwrap_or(false) {
            changed.push("id_tour".to_string());
        }
        if request.id_agencia.as_ref().map(|a| *a != old.id_agencia).unwrap_or(false) {
            changed.push("id_agencia".to_string());
        }
        if request.status.as_ref().map(|s| s != &old.status).unwrap_or(false) {
            changed.push("status".to_string());
        }
        if request.nro_pasajeros.as_ref().map(|n| *n != old.nro_pasajeros).unwrap_or(false) {
            changed.push("nro_pasajeros".to_string());
        }
        
        changed
    }
}
