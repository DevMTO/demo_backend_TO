//! CadenaHotelera Service - Lógica de negocio para cadenas hoteleras

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateCadenaHoteleraRequest, UpdateCadenaHoteleraRequest, CadenaHoteleraResponse, CadenaHoteleraListItemDto,
};
use crate::application::ports::{
    CadenaHoteleraRepositoryPort, NotificationServicePort,
};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    CadenaHotelera, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

pub struct CadenaHoteleraService {
    cadena_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl CadenaHoteleraService {
    pub fn new(
        cadena_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            cadena_repository,
            logging_service,
            notification_service,
        }
    }

    #[instrument(skip(self))]
    pub async fn list_cadenas(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CadenaHoteleraListItemDto>, i64), ApplicationError> {
        let offset = (page - 1) * page_size;
        let (items, total) = self.cadena_repository
            .list_with_encargado(page_size, offset)
            .await?;
        info!("Listadas {} cadenas hoteleras de {} total", items.len(), total);
        Ok((items, total))
    }

    #[instrument(skip(self))]
    pub async fn get_cadena(&self, id: i32) -> Result<CadenaHoteleraResponse, ApplicationError> {
        let cadena = self.cadena_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)))?;
        
        info!("Cadena hotelera encontrada: {} (ID: {})", cadena.nombre, id);
        Ok(CadenaHoteleraResponse::from(cadena))
    }

    #[instrument(skip(self, request))]
    pub async fn create_cadena(
        &self,
        request: CreateCadenaHoteleraRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<CadenaHoteleraResponse, ApplicationError> {
        let cadena = request.into_entity(Some(created_by));
        
        let created = self.cadena_repository.create(&cadena).await?;
        info!("Cadena hotelera creada: {} (ID: {})", created.nombre, created.id);
        
        if let Err(e) = self.logging_service.log_create::<CadenaHotelera>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::CadenaHotelera,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de cadena hotelera: {}", e);
        }
        
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nueva cadena hotelera creada",
            &format!("{} ha creado la cadena hotelera '{}'", username, created.nombre),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de cadena hotelera creada: {}", e);
        }

        Ok(CadenaHoteleraResponse::from(created))
    }

    #[instrument(skip(self, request))]
    pub async fn update_cadena(
        &self,
        id: i32,
        request: UpdateCadenaHoteleraRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<CadenaHoteleraResponse, ApplicationError> {
        let old_cadena = self.cadena_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)))?;
        
        let changed_fields = self.detect_changed_fields(&old_cadena, &request);
        let updated_entity = request.apply_to(old_cadena.clone(), Some(updated_by));
        
        let result = self.cadena_repository.update(&updated_entity).await?;
        info!("Cadena hotelera actualizada: {} (ID: {})", result.nombre, result.id);
        
        if let Err(e) = self.logging_service.log_update::<CadenaHotelera>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::CadenaHotelera,
            id,
            Some(&old_cadena),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de cadena hotelera: {}", e);
        }
        
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Cadena hotelera actualizada",
            &format!("{} ha actualizado la cadena hotelera '{}'", username, result.nombre),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de cadena hotelera actualizada: {}", e);
        }

        Ok(CadenaHoteleraResponse::from(result))
    }

    #[instrument(skip(self))]
    pub async fn delete_cadena(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        let cadena = self.cadena_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)))?;
        
        let deleted = self.cadena_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)));
        }
        
        info!("[DELETE] Cadena hotelera desactivada: {} (ID: {})", cadena.nombre, id);
        
        if let Err(e) = self.logging_service.log_delete::<CadenaHotelera>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::CadenaHotelera,
            id,
            Some(&cadena),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de cadena hotelera: {}", e);
        }
        
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Cadena hotelera desactivada",
            &format!("{} ha desactivado la cadena hotelera '{}'", username, cadena.nombre),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de cadena hotelera desactivada: {}", e);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn restore_cadena(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<CadenaHoteleraResponse, ApplicationError> {
        let restored = self.cadena_repository.restore(id, restored_by).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)));
        }
        
        let cadena = self.cadena_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)))?;
        
        info!("Cadena hotelera restaurada: {} (ID: {})", cadena.nombre, id);
        
        if let Err(e) = self.logging_service.log_update::<CadenaHotelera>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::CadenaHotelera,
            id,
            None::<&CadenaHotelera>,
            Some(&cadena),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de cadena hotelera: {}", e);
        }
        
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Cadena hotelera restaurada",
            &format!("{} ha restaurado la cadena hotelera '{}'", username, cadena.nombre),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de cadena hotelera restaurada: {}", e);
        }

        Ok(CadenaHoteleraResponse::from(cadena))
    }

    #[instrument(skip(self))]
    pub async fn hard_delete_cadena(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        let cadena = self.cadena_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)))?;
        
        if !self.cadena_repository.hard_delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id)));
        }
        
        info!("[HARD_DELETE] Cadena hotelera eliminada permanentemente: {} (ID: {})", cadena.nombre, id);
        
        if let Err(e) = self.logging_service.log_delete::<CadenaHotelera>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::CadenaHotelera,
            id,
            Some(&cadena),
            Some("HARD_DELETE - Eliminacion permanente".to_string()),
        ).await {
            warn!("Error al registrar log de hard_delete de cadena hotelera: {}", e);
        }
        
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "Cadena hotelera ELIMINADA permanentemente",
            &format!("{} ha eliminado permanentemente la cadena hotelera '{}' (ID: {})", username, cadena.nombre, id),
            NotificationType::Warning,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de cadena hotelera eliminada: {}", e);
        }
        
        Ok(())
    }

    /// Obtener la cadena hotelera del usuario actual (para rol HotelesGerente)
    #[instrument(skip(self))]
    pub async fn get_mi_cadena(
        &self,
        user_role: &UserRole,
        id_entidad: Option<i32>,
        id_persona: Option<i32>,
        username: &str,
    ) -> Result<CadenaHoteleraResponse, ApplicationError> {
        info!("Buscando cadena hotelera para usuario '{}' (id_persona: {:?}, id_entidad: {:?}, role: {:?})", 
            username, id_persona, id_entidad, user_role);
        
        let mut cadena: Option<CadenaHotelera> = None;
        
        let is_cadena_user = *user_role == UserRole::HotelesGerenteCadena;
        
        if is_cadena_user {
            if let Some(entity_id) = id_entidad {
                cadena = self.cadena_repository
                    .find_by_id(entity_id)
                    .await?;
                if cadena.is_some() {
                    info!("Cadena hotelera encontrada por id_entidad: {}", entity_id);
                }
            }
        }
        
        if cadena.is_none() {
            if let Some(persona_id) = id_persona {
                cadena = self.cadena_repository
                    .find_by_encargado(persona_id)
                    .await?;
                if cadena.is_some() {
                    info!("Cadena hotelera encontrada por encargado (persona_id: {})", persona_id);
                }
            }
        }
        
        match cadena {
            Some(c) => Ok(CadenaHoteleraResponse::from(c)),
            None => {
                info!("Usuario '{}' no tiene cadena hotelera asociada", username);
                Err(ApplicationError::NotFound("No tienes una cadena hotelera asociada".to_string()))
            }
        }
    }

    fn detect_changed_fields(&self, old: &CadenaHotelera, request: &UpdateCadenaHoteleraRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.nombre.as_ref().map(|n| n != &old.nombre).unwrap_or(false) {
            changed.push("nombre".to_string());
        }
        if request.telefono.as_ref().map(|t| Some(t.clone()) != old.telefono).unwrap_or(false) {
            changed.push("telefono".to_string());
        }
        if request.correo.as_ref().map(|c| Some(c.clone()) != old.correo).unwrap_or(false) {
            changed.push("correo".to_string());
        }
        if request.encargado.is_some() && request.encargado != old.encargado {
            changed.push("encargado".to_string());
        }
        if request.media.is_some() {
            changed.push("media".to_string());
        }
        if request.is_active.as_ref().map(|a| *a != old.is_active).unwrap_or(false) {
            changed.push("is_active".to_string());
        }
        if request.paleta_colores.is_some() {
            changed.push("paleta_colores".to_string());
        }
        
        changed
    }
}
