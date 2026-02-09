//! Pago Service - Lógica de negocio para pagos
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con pagos:
//! - Registro de pagos (con validación de file existente)
//! - Actualización de pagos
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreatePagoRequest, UpdatePagoRequest, PagoResponse,
};
use crate::application::ports::{PagoRepositoryPort, FileRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Pago, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de pagos - contiene la lógica de negocio
pub struct PagoService {
    pago_repository: Arc<dyn PagoRepositoryPort>,
    file_repository: Arc<dyn FileRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl PagoService {
    pub fn new(
        pago_repository: Arc<dyn PagoRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            pago_repository,
            file_repository,
            logging_service,
            notification_service,
        }
    }

    /// Listar pagos con paginación
    #[instrument(skip(self))]
    pub async fn list_pagos(
        &self,
        options: PaginationOptions,
    ) -> Result<(Vec<PagoResponse>, i64, i64), ApplicationError> {
        let result = self.pago_repository
            .list_paginated(options)
            .await?;
        
        let total = result.total;
        let pages = result.pages();
        let current_page = result.current_page();
        let items: Vec<PagoResponse> = result.data.into_iter().map(Into::into).collect();
        info!("Listados {} pagos (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener pago por ID
    #[instrument(skip(self))]
    pub async fn get_pago(&self, id: i32) -> Result<PagoResponse, ApplicationError> {
        let pago = self.pago_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))?;
        
        let response = PagoResponse::from(pago.clone());
        info!("Pago encontrado: {} {} (ID: {})", pago.tipo_movimiento, pago.monto, id);
        
        Ok(response)
    }

    /// Registrar un nuevo pago
    /// 
    /// # Validaciones de negocio:
    /// - El file asociado debe existir
    #[instrument(skip(self, request))]
    pub async fn register_pago(
        &self,
        request: CreatePagoRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<PagoResponse, ApplicationError> {
        // Validación de negocio: el file debe existir
        let _ = self.file_repository
            .find_by_id(request.id_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        
        // Crear entidad de dominio
        let pago = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.pago_repository.create(&pago).await?;
        info!("Pago registrado: {} ${} (ID: {})", created.tipo_movimiento, created.monto, created.id);
        
        // Logging del evento
        let description = format!("{} ${}", created.tipo_movimiento, created.monto);
        if let Err(e) = self.logging_service.log_create::<Pago>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Pago,
            created.id,
            &description,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de pago: {}", e);
        }
        
        // Notificación a admins (pagos son importantes)
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Pago registrado",
            &format!("{} ha registrado {} de ${} en file #{}", username, created.tipo_movimiento, created.monto, created.id_file),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de pago registrado: {}", e);
        }
        
        Ok(PagoResponse::from(created))
    }

    /// Actualizar un pago existente
    #[instrument(skip(self, request))]
    pub async fn update_pago(
        &self,
        id: i32,
        request: UpdatePagoRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<PagoResponse, ApplicationError> {
        // Verificar que existe
        let old_pago = self.pago_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_pago, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_pago.clone(), Some(updated_by));
        
        // Persistir
        let result = self.pago_repository.update(&updated_entity).await?;
        info!("✏️ Pago actualizado: {} ${} (ID: {})", result.tipo_movimiento, result.monto, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Pago>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Pago,
            id,
            Some(&old_pago),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de pago: {}", e);
        }
        
        // Notificación si hubo cambios importantes
        if !changed_fields.is_empty() {
            let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
            if let Err(e) = self.notification_service.notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "Pago modificado",
                &format!("{} ha modificado pago #{} ${} (campos: {})", username, id, result.monto, changed_fields.join(", ")),
                NotificationType::Warning,
                NotificationCategory::Crud,
                NotificationPriority::High,
                Some(updated_by),
            ).await {
                warn!("Error al enviar notificación de pago modificado: {}", e);
            }
        }
        
        Ok(PagoResponse::from(result))
    }

    /// Eliminar un pago
    #[instrument(skip(self))]
    pub async fn delete_pago(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener pago antes de eliminar
        let pago = self.pago_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))?;
        
        // Eliminar
        let deleted = self.pago_repository.delete(id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Pago {} no encontrado", id)));
        }
        
        info!("[DELETE] Pago eliminado: {} ${} (ID: {})", pago.tipo_movimiento, pago.monto, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Pago>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Pago,
            id,
            Some(&pago),
            None,
        ).await {
            warn!("Error al registrar log de eliminación de pago: {}", e);
        }
        
        // Notificación a admins (eliminar pagos es crítico)
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Pago eliminado",
            &format!("{} ha eliminado pago #{} ({} ${}) del file #{}", username, id, pago.tipo_movimiento, pago.monto, pago.id_file),
            NotificationType::Error,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de pago eliminado: {}", e);
        }
        
        Ok(())
    }

    /// Listar pagos por file
    #[instrument(skip(self))]
    pub async fn list_pagos_by_file(&self, file_id: i32) -> Result<Vec<PagoResponse>, ApplicationError> {
        let pagos = self.pago_repository
            .find_by_file(file_id)
            .await?;
        
        info!("{} pagos encontrados para file {}", pagos.len(), file_id);
        Ok(pagos.into_iter().map(Into::into).collect())
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &Pago, request: &UpdatePagoRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.tipo_movimiento.as_ref().map(|t| t != &old.tipo_movimiento).unwrap_or(false) {
            changed.push("tipo_movimiento".to_string());
        }
        if request.concepto.as_ref().map(|c| c != &old.concepto).unwrap_or(false) {
            changed.push("concepto".to_string());
        }
        if request.metodo_pago.as_ref().map(|m| Some(m.clone()) != old.metodo_pago).unwrap_or(false) {
            changed.push("metodo_pago".to_string());
        }
        if request.notas.as_ref().map(|n| Some(n.clone()) != old.notas).unwrap_or(false) {
            changed.push("notas".to_string());
        }
        
        changed
    }
}
