//! File Service - Lógica de negocio para archivos de viaje (Files)
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con files:
//! - Creación de files
//! - Actualización de files
//! - Búsqueda
//! - Logging de actividades
//! - Notificaciones
//! - Gestión de tours asociados (file_tours)

use std::sync::Arc;
use chrono::NaiveDate;
use bigdecimal::BigDecimal;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateFileRequest, UpdateFileRequest, FileResponse, FileTourDto,
};
use crate::application::ports::{FileRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::ports::{FileTourRepositoryPort, FileTourInputData};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    File, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de files - contiene la lógica de negocio
pub struct FileService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl FileService {
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
            logging_service,
            notification_service,
        }
    }

    /// Carga los tours de un file con información completa del tour (INNER JOIN) y los convierte a DTO
    async fn load_file_tours(&self, file_id: i32) -> Result<Vec<FileTourDto>, ApplicationError> {
        let tours = self.file_tour_repository.find_by_file_with_tour(file_id).await?;
        Ok(tours.into_iter().map(|t| FileTourDto {
            id: t.id,
            id_tour: t.id_tour,
            orden: t.orden,
            precio_aplicado: t.precio_aplicado.clone(),
            notas: t.notas,
            fecha_tour: t.fecha_tour,
            // Campos de recojo movidos desde files
            turno_tour: t.turno_tour,
            lugar_recojo: t.lugar_recojo,
            hora_recojo: t.hora_recojo,
            // Convertir JsonValue a GeoLocation
            geo_recojo: t.geo_recojo.and_then(|v| serde_json::from_value(v).ok()),
            // Estado del file_tour
            status: t.status,
            // Información completa del tour (INNER JOIN)
            tour_nombre: Some(t.tour_nombre),
            tour_lugar_inicio: t.tour_lugar_inicio,
            tour_lugar_fin: t.tour_lugar_fin,
            tour_precio_base: Some(t.tour_precio_base),
            tour_duracion_dias: t.tour_duracion_dias,
            tour_tipo: t.tour_tipo,
            tour_is_active: Some(t.tour_is_active),
        }).collect())
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
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in result.data {
            let tours = self.load_file_tours(file.id).await?;
            items.push(FileResponse::from_file_with_tours(file, tours));
        }
        
        info!("Listados {} files (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener file por ID
    #[instrument(skip(self))]
    pub async fn get_file(&self, id: i32) -> Result<FileResponse, ApplicationError> {
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        let tours = self.load_file_tours(id).await?;
        
        info!("File encontrado: ID {} con {} tours", id, tours.len());
        Ok(FileResponse::from_file_with_tours(file, tours))
    }

    /// Crear un nuevo file
    /// 
    /// Si el usuario tiene rol "Agencias", se usa su id_entidad como id_agencia automáticamente.
    /// Si el usuario es SuperAdmin o Admin, debe proporcionar id_agencia en el request.
    #[instrument(skip(self, request))]
    pub async fn create_file(
        &self,
        request: CreateFileRequest,
        created_by: i32,
        created_by_username: Option<String>,
        user_role: UserRole,
        user_id_entidad: Option<i32>,
    ) -> Result<FileResponse, ApplicationError> {
        // Resolver id_agencia según el rol del usuario
        let id_agencia_resolved = match user_role {
            UserRole::Agencias => {
                // Para agencias, usar su id_entidad automáticamente
                user_id_entidad.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Usuario de agencia sin id_entidad configurado".to_string()
                    )
                })?
            },
            _ => {
                // Para superadmin/admin, debe venir en el request
                request.id_agencia.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Debe seleccionar una agencia para crear el file".to_string()
                    )
                })?
            }
        };

        // Obtener tours del request antes de consumirlo
        let tours_input = request.get_tours();
        if tours_input.is_empty() {
            return Err(ApplicationError::Validation(
                "Debe especificar al menos un tour para el file".to_string()
            ));
        }

        // Crear entidad de dominio con id_agencia resuelto
        let file = request.into_entity(Some(created_by), id_agencia_resolved);
        
        // Persistir el file
        let created = self.file_repository.create(&file).await?;
        info!("File creado: ID {} para fechas {} - {}", created.id, created.fecha_inicio, created.fecha_fin);
        
        // Insertar tours asociados (con fecha_tour y campos de recojo)
        let tours_data: Vec<FileTourInputData> = tours_input
            .into_iter()
            .enumerate()
            .map(|(idx, t)| {
                let orden = t.orden.unwrap_or((idx + 1) as i32);
                let precio = t.precio_aplicado.map(|p| BigDecimal::try_from(p).unwrap_or_default());
                // Convertir GeoLocation a JsonValue para la BD
                let geo_recojo_json = t.geo_recojo.and_then(|g| {
                    if g.has_data() {
                        serde_json::to_value(g).ok()
                    } else {
                        None
                    }
                });
                FileTourInputData {
                    id_tour: t.id_tour,
                    orden,
                    precio_aplicado: precio,
                    notas: t.notas,
                    fecha_tour: t.fecha_tour,
                    turno_tour: t.turno_tour,
                    lugar_recojo: t.lugar_recojo,
                    hora_recojo: t.hora_recojo,
                    status: t.status,
                    geo_recojo: geo_recojo_json,
                }
            })
            .collect();
        
        let _created_tours = self.file_tour_repository
            .add_many(created.id, tours_data, Some(created_by))
            .await?;
        info!("{} tours asignados al file {}", _created_tours.len(), created.id);
        
        // Cargar tours con info completa (JOIN) para el response
        let tours_dto = self.load_file_tours(created.id).await?;
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<File>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::File,
            created.id,
            &format!("File #{} - {} a {} ({} tours)", created.id, created.fecha_inicio, created.fecha_fin, tours_dto.len()),
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de file: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo file creado",
            &format!("{} ha creado el file #{} con {} tours", username, created.id, tours_dto.len()),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de file creado: {}", e);
        }
        
        Ok(FileResponse::from_file_with_tours(created, tours_dto))
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
        let mut changed_fields = self.detect_changed_fields(&old_file, &request);
        
        // Obtener tours del request antes de consumirlo
        let tours_update = request.get_tours();
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_file.clone(), Some(updated_by));
        
        // Persistir file
        let result = self.file_repository.update(&updated_entity).await?;
        info!("✏️ File actualizado: ID {}", result.id);
        
        // Actualizar tours si se especificaron
        let tours_dto = if let Some(tours_input) = tours_update {
            // Eliminar todos los tours existentes
            self.file_tour_repository.remove_by_file(id).await?;
            
            // Insertar nuevos tours (con fecha_tour y campos de recojo)
            let tours_data: Vec<FileTourInputData> = tours_input
                .into_iter()
                .enumerate()
                .map(|(idx, t)| {
                    let orden = t.orden.unwrap_or((idx + 1) as i32);
                    let precio = t.precio_aplicado.map(|p| BigDecimal::try_from(p).unwrap_or_default());
                    // Convertir GeoLocation a JsonValue para la BD
                    let geo_recojo_json = t.geo_recojo.and_then(|g| {
                        if g.has_data() {
                            serde_json::to_value(g).ok()
                        } else {
                            None
                        }
                    });
                    FileTourInputData {
                        id_tour: t.id_tour,
                        orden,
                        precio_aplicado: precio,
                        notas: t.notas,
                        fecha_tour: t.fecha_tour,
                        turno_tour: t.turno_tour,
                        lugar_recojo: t.lugar_recojo,
                        hora_recojo: t.hora_recojo,
                        status: t.status,
                        geo_recojo: geo_recojo_json,
                    }
                })
                .collect();
            
            let created_tours = self.file_tour_repository
                .add_many(id, tours_data, Some(updated_by))
                .await?;
            
            changed_fields.push("tours".to_string());
            info!("Tours actualizados para file {}: {} tours", id, created_tours.len());
            
            // Cargar con JOIN para info completa
            self.load_file_tours(id).await?
        } else {
            // Cargar tours existentes
            self.load_file_tours(id).await?
        };
        
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
            warn!("Error al registrar log de actualización de file: {}", e);
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
                warn!("Error al enviar notificación de file actualizado: {}", e);
            }
        }
        
        Ok(FileResponse::from_file_with_tours(result, tours_dto))
    }

    /// Desactivar un file (soft delete)
    #[instrument(skip(self))]
    pub async fn delete_file(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener file antes de desactivar
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Soft delete
        let deleted = self.file_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        info!("[DELETE] File desactivado: ID {}", id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<File>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::File,
            id,
            Some(&file),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de file: {}", e);
        }
        
        // Notificación a admins
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File desactivado",
            &format!("{} ha desactivado el file #{} (fechas: {} - {})", username, id, file.fecha_inicio, file.fecha_fin),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de file desactivado: {}", e);
        }
        
        Ok(())
    }

    /// Restaurar un file desactivado
    #[instrument(skip(self))]
    pub async fn restore_file(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Restore via repository
        if !self.file_repository.restore(id, restored_by).await? {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        // Obtener file restaurado
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        info!("♻️ File restaurado: ID {}", id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::File,
            id,
            None,
            Some(&file),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de file: {}", e);
        }
        
        // Notificación a admins - Success
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File restaurado",
            &format!("{} ha restaurado el file #{} (fechas: {} - {})", username, id, file.fecha_inicio, file.fecha_fin),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de file restaurado: {}", e);
        }
        
        Ok(())
    }

    /// Eliminación permanente de file (hard delete) - Solo SuperAdmin
    #[instrument(skip(self))]
    pub async fn hard_delete_file(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener file antes de eliminar para el log
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Eliminar permanentemente
        let deleted = self.file_repository.hard_delete(id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        info!("🗑️ File ELIMINADO PERMANENTEMENTE: ID {}", id);
        
        // Logging del evento (acción crítica)
        if let Err(e) = self.logging_service.log_delete::<File>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::File,
            id,
            Some(&file),
            Some("HARD_DELETE - Eliminación permanente".to_string()),
        ).await {
            warn!("Error al registrar log de eliminación permanente de file: {}", e);
        }
        
        // Notificación CRÍTICA a SuperAdmin únicamente
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "⚠️ FILE ELIMINADO PERMANENTEMENTE",
            &format!(
                "ACCIÓN CRÍTICA: {} ha eliminado PERMANENTEMENTE el file #{} (fechas: {} - {}). Esta acción NO se puede deshacer.",
                username, id, file.fecha_inicio, file.fecha_fin
            ),
            NotificationType::Error,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de eliminación permanente de file: {}", e);
        }
        
        Ok(())
    }

    /// Listar files por agencia
    #[instrument(skip(self))]
    pub async fn list_files_by_agencia(&self, agencia_id: i32) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_by_agencia(agencia_id)
            .await?;
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in files {
            let tours = self.load_file_tours(file.id).await?;
            items.push(FileResponse::from_file_with_tours(file, tours));
        }
        
        info!("{} files encontrados para agencia {} (con tours cargados)", items.len(), agencia_id);
        Ok(items)
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
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in files {
            let tours = self.load_file_tours(file.id).await?;
            items.push(FileResponse::from_file_with_tours(file, tours));
        }
        
        info!("{} files encontrados entre {} y {} (con tours cargados)", items.len(), from, to);
        Ok(items)
    }

    /// Listar files próximos (en los próximos 7 días)
    #[instrument(skip(self))]
    pub async fn list_files_upcoming(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_upcoming()
            .await?;
        
        info!("{} files próximos encontrados", files.len());
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Listar files con pago pendiente
    #[instrument(skip(self))]
    pub async fn list_files_pending_payment(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_pending_payment()
            .await?;
        
        info!("{} files con pago pendiente encontrados", files.len());
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
        // id_tour eliminado - tours ahora están en file_tours
        if request.tours.is_some() {
            changed.push("tours".to_string());
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
