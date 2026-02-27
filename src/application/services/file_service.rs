//! File Service - Lógica de negocio para archivos de viaje (Files)
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con files:
//! - Creación de files
//! - Actualización de files
//! - Búsqueda
//! - Logging de actividades
//! - Notificaciones
//! - Gestión de tours asociados (file_tours)
//! - **Confirmación de reservas con creación de pagos pendientes**

use std::sync::Arc;
use chrono::{NaiveDate, Datelike, Duration, Utc};
use bigdecimal::BigDecimal;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateFileRequest, UpdateFileRequest, FileResponse, FileTourDto,
    ConfirmReservaRequest, ConfirmReservaResponse,
};
use crate::application::ports::{FileRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::ports::{FileTourRepositoryPort, FileTourInputData, PagoFileRepositoryPort, AgenciaRepositoryPort};
use crate::application::ports::{FileEntradaRepositoryPort, EntradaPrecioRepositoryPort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    File, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::NewPagoFileModel;

/// Servicio de files - contiene la lógica de negocio
pub struct FileService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
    pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
}

impl FileService {
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
        entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
            logging_service,
            notification_service,
            pago_file_repository,
            agencia_repository,
            file_entrada_repository,
            entrada_precio_repository,
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
            turno_tour: t.turno_tour,
            lugar_recojo: t.lugar_recojo,
            hora_recojo: t.hora_recojo,
            geo_recojo: t.geo_recojo.and_then(|v| serde_json::from_value(v).ok()),
            status: t.status,
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
        let tours_len = tours.len();
        
        let response = FileResponse::from_file_with_tours(file, tours);
        info!("File encontrado: ID {} con {} tours", id, tours_len);
        
        Ok(response)
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
        
        // NOTE: Deliberadamente se omitido 'Notificación a admins'
        
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

    // =========================================================================
    // CONFIRMACIÓN DE RESERVA
    // =========================================================================

    /// Confirmar una reserva (file)
    /// 
    /// Este método:
    /// 1. Verifica que el file exista y esté en estado "reservado"
    /// 2. Actualiza el status a "confirmado"
    /// 3. Crea un registro en pagos_files con estado "pendiente"
    /// 4. Notifica a los admins
    /// 5. Registra en el log de actividad
    #[instrument(skip(self))]
    pub async fn confirmar_reserva(
        &self,
        request: ConfirmReservaRequest,
        confirmed_by: i32,
        confirmed_by_username: Option<String>,
    ) -> Result<ConfirmReservaResponse, ApplicationError> {
        // 1. Obtener el file
        let file = self.file_repository
            .find_by_id(request.file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(
                format!("File {} no encontrado", request.file_id)
            ))?;
        
        // 2. Verificar que el status sea válido para confirmar
        let valid_statuses = ["pendiente", "reservado"];
        if !valid_statuses.contains(&file.status.as_str()) {
            return Err(ApplicationError::Validation(format!(
                "El file no puede ser confirmado. Estado actual: '{}'. Estados válidos para confirmación: pendiente, reservado",
                file.status
            )));
        }
        
        // Verificar que no existan ya pagos_file para este file
        let existing_pagos = self.pago_file_repository.find_all_by_file(request.file_id).await?;
        if !existing_pagos.is_empty() {
            return Err(ApplicationError::Validation(
                "Este file ya tiene registros de pago asociados".to_string()
            ));
        }
        
        // 3. Obtener info de la agencia
        let agencia = self.agencia_repository
            .find_by_id(file.id_agencia)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(
                format!("Agencia {} no encontrada", file.id_agencia)
            ))?;
        
        // 4. Calcular montos y fechas
        let monto_total = request.monto_total
            .map(|m| BigDecimal::try_from(m).unwrap_or_default())
            .unwrap_or_else(|| file.monto_total.clone());
        
        // Calcular fecha_vencimiento según política de pago de la agencia
        let fecha_vencimiento = if agencia.pago_anticipado {
            // Pago anticipado: vence 1 día antes de la fecha del primer tour del file
            let tours = self.file_tour_repository.find_by_file_with_tour(request.file_id).await?;
            let earliest_tour_date = tours.iter()
                .filter_map(|t| t.fecha_tour)
                .min();
            
            match earliest_tour_date {
                Some(fecha) => {
                    // 1 día antes de la fecha del tour más temprano
                    fecha - chrono::Duration::days(1)
                },
                None => {
                    // Sin tours, usar 7 días desde ahora como fallback
                    warn!("⚠️ Agencia con pago_anticipado pero file {} sin tours con fecha", request.file_id);
                    (Utc::now() + Duration::days(7)).date_naive()
                }
            }
        } else {
            // No es pago anticipado: vence el dia (1ro del mes + dias_pago_anticipado)
            let dias = agencia.dias_pago_anticipado.unwrap_or(30);
            let now = Utc::now().date_naive();
            // Primer día del mes actual
            let primer_dia_mes = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .unwrap_or(now);
            primer_dia_mes + chrono::Duration::days(dias as i64)
        };
        
        // 5. Actualizar el file a status "confirmado"
        let mut updated_file = file.clone();
        updated_file.status = "confirmado".to_string();
        updated_file.monto_total = monto_total.clone();
        updated_file.updated_by = Some(confirmed_by);
        updated_file.updated_at = Utc::now();
        
        let updated_file = self.file_repository.update(&updated_file).await?;
        info!("✅ File {} confirmado", request.file_id);
        
        // 6. Crear un pago_file (deuda) POR CADA file_tour del file
        let tours = self.file_tour_repository.find_by_file_with_tour(request.file_id).await?;
        
        if tours.is_empty() {
            return Err(ApplicationError::Validation(
                "El file no tiene tours asociados. No se puede confirmar sin file_tours.".to_string()
            ));
        }
        
        let mut pago_file_ids: Vec<i32> = Vec::new();
        
        for ft in &tours {
            // Calcular monto del tour: precio_aplicado o tour_precio_base * nro_pasajeros
            let monto_tour = ft.precio_aplicado.clone()
                .unwrap_or_else(|| ft.tour_precio_base.clone());
            
            // Calcular monto de entradas para este file_tour
            let entradas_ft = self.file_entrada_repository.find_by_file_tour(ft.id).await.unwrap_or_default();
            let zero = BigDecimal::from(0);
            let mut monto_entradas = zero.clone();
            let tiene_entradas = !entradas_ft.is_empty();
            
            for fe in &entradas_ft {
                if let Some(precio_id) = fe.id_entrada_precio {
                    if let Ok(Some(precio)) = self.entrada_precio_repository.find_by_id(precio_id).await {
                        monto_entradas += &precio.precio * BigDecimal::from(fe.cantidad);
                    }
                }
            }
            
            let new_pago = NewPagoFileModel {
                id_file: request.file_id,
                id_agencia: file.id_agencia,
                monto_total: monto_tour,
                monto_pagado: zero.clone(),
                estado: "pendiente",
                fecha_vencimiento: Some(fecha_vencimiento),
                notas: request.notas.as_deref(),
                created_by: Some(confirmed_by),
                id_file_tour: Some(ft.id),
                tipo_registro: "deuda",
                monto_saldo_favor: None,
                saldo_autorizado: false,
                saldo_autorizado_por: None,
                saldo_autorizado_at: None,
                entradas: tiene_entradas,
                entrada_precio: if tiene_entradas { Some(monto_entradas) } else { None },
                cuota: Some(0),
            };
            
            let pago = self.pago_file_repository.create(new_pago).await?;
            info!("💰 Deuda por tour creada: pago_file ID {} para file_tour {} (file {})", pago.id, ft.id, request.file_id);
            pago_file_ids.push(pago.id);
        }
        
        info!("💰 {} deudas creadas para file {}", pago_file_ids.len(), request.file_id);
        
        // 7. Registrar en el log de actividad
        let username = confirmed_by_username.clone().unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(confirmed_by),
            confirmed_by_username.clone(),
            EntityType::File,
            request.file_id,
            Some(&file),
            Some(&updated_file),
            Some(vec!["status".to_string(), "confirmacion".to_string()]),
            None, // IP no aplica en operación de servicio
        ).await {
            warn!("Error al registrar log de confirmación: {}", e);
        }
        
        // 8. Notificar a los admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "📋 Reserva Confirmada",
            &format!(
                "{} ha confirmado la reserva #{} (File #{}).\nAgencia: {}\nMonto total: S/ {}\nVencimiento de pago: {}",
                username,
                updated_file.file_code.clone().unwrap_or_else(|| format!("F-{}", request.file_id)),
                request.file_id,
                agencia.nombre,
                monto_total,
                fecha_vencimiento
            ),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(confirmed_by),
        ).await {
            warn!("Error al enviar notificación de confirmación: {}", e);
        }
        
        // 9. Notificar también al contador de la agencia específica (filtrado por id_entidad)
        // Solo notificará a los usuarios AgenciasContador que pertenezcan a ESTA agencia
        if let Err(e) = self.notification_service.notify_roles_for_entity(
            vec![UserRole::AgenciasContador, UserRole::AgenciasGerente],
            file.id_agencia, // Filtrar por la agencia del file
            "💰 Nuevo pago pendiente",
            &format!(
                "Se ha confirmado la reserva #{} con un monto de S/ {}.\nFecha de vencimiento: {}\nPor favor, gestione el pago.",
                updated_file.file_code.clone().unwrap_or_else(|| format!("F-{}", request.file_id)),
                monto_total,
                fecha_vencimiento
            ),
            NotificationType::Warning,
            NotificationCategory::Financial,
            NotificationPriority::High,
            Some(confirmed_by),
        ).await {
            warn!("Error al enviar notificación al contador de la agencia: {}", e);
        }
        
        // 10. Cargar tours para el response
        let tours_dto = self.load_file_tours(request.file_id).await?;
        
        let num_deudas = pago_file_ids.len();
        let mensaje = format!(
            "Reserva confirmada exitosamente. Se han generado {} deudas pendientes (una por tour) con monto total S/ {} y vencimiento el {}",
            num_deudas, &monto_total, fecha_vencimiento
        );
        
        Ok(ConfirmReservaResponse {
            file: FileResponse::from_file_with_tours(updated_file, tours_dto),
            pago_file_ids,
            monto_total,
            fecha_vencimiento: fecha_vencimiento.to_string(),
            mensaje,
        })
    }
}
