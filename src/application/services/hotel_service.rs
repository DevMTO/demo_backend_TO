//! Hotel Service - Lógica de negocio para hoteles

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateHotelRequest, UpdateHotelRequest, HotelResponse, HotelListItemDto,
};
use crate::application::ports::{
    HotelRepositoryPort, CadenaHoteleraRepositoryPort, NotificationServicePort,
};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Hotel, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

pub struct HotelService {
    hotel_repository: Arc<dyn HotelRepositoryPort>,
    cadena_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl HotelService {
    pub fn new(
        hotel_repository: Arc<dyn HotelRepositoryPort>,
        cadena_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            hotel_repository,
            cadena_repository,
            logging_service,
            notification_service,
        }
    }

    #[instrument(skip(self))]
    pub async fn list_hoteles(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<HotelListItemDto>, i64), ApplicationError> {
        let offset = (page - 1) * page_size;
        let (items, total) = self.hotel_repository
            .list_with_cadena(page_size, offset)
            .await?;
        info!("Listados {} hoteles de {} total", items.len(), total);
        Ok((items, total))
    }

    #[instrument(skip(self))]
    pub async fn list_hoteles_by_cadena(
        &self,
        id_cadena: i32,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Hotel>, i64), ApplicationError> {
        let offset = (page - 1) * page_size;
        let items = self.hotel_repository
            .list_by_cadena(id_cadena, page_size, offset)
            .await?;
        let total = self.hotel_repository
            .count_by_cadena(id_cadena)
            .await?;
        info!("Listados {} hoteles de cadena {} de {} total", items.len(), id_cadena, total);
        Ok((items, total))
    }

    #[instrument(skip(self))]
    pub async fn get_hotel(&self, id: i32) -> Result<HotelResponse, ApplicationError> {
        let hotel = self.hotel_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id)))?;
        
        info!("Hotel encontrado: {} (ID: {})", hotel.nombre, id);
        Ok(HotelResponse::from(hotel))
    }

    #[instrument(skip(self, request))]
    pub async fn create_hotel(
        &self,
        request: CreateHotelRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<HotelResponse, ApplicationError> {
        // Verificar que la cadena existe
        self.cadena_repository
            .find_by_id(request.id_cadena)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", request.id_cadena)))?;

        let hotel = request.into_entity(Some(created_by));
        
        let created = self.hotel_repository.create(&hotel).await?;
        info!("Hotel creado: {} (ID: {})", created.nombre, created.id);
        
        if let Err(e) = self.logging_service.log_create::<Hotel>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Hotel,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de hotel: {}", e);
        }
        
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo hotel creado",
            &format!("{} ha creado el hotel '{}'", username, created.nombre),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de hotel creado: {}", e);
        }

        Ok(HotelResponse::from(created))
    }

    #[instrument(skip(self, request))]
    pub async fn update_hotel(
        &self,
        id: i32,
        request: UpdateHotelRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<HotelResponse, ApplicationError> {
        let old_hotel = self.hotel_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id)))?;
        
        // Si se cambia la cadena, verificar que existe
        if let Some(new_cadena_id) = request.id_cadena {
            if new_cadena_id != old_hotel.id_cadena {
                self.cadena_repository
                    .find_by_id(new_cadena_id)
                    .await?
                    .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", new_cadena_id)))?;
            }
        }
        
        let changed_fields = self.detect_changed_fields(&old_hotel, &request);
        let updated_entity = request.apply_to(old_hotel.clone(), Some(updated_by));
        
        let result = self.hotel_repository.update(&updated_entity).await?;
        info!("Hotel actualizado: {} (ID: {})", result.nombre, result.id);
        
        if let Err(e) = self.logging_service.log_update::<Hotel>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Hotel,
            id,
            Some(&old_hotel),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de hotel: {}", e);
        }
        
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Hotel actualizado",
            &format!("{} ha actualizado el hotel '{}'", username, result.nombre),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de hotel actualizado: {}", e);
        }

        Ok(HotelResponse::from(result))
    }

    #[instrument(skip(self))]
    pub async fn delete_hotel(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        let hotel = self.hotel_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id)))?;
        
        let deleted = self.hotel_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Hotel {} no encontrado", id)));
        }
        
        info!("[DELETE] Hotel desactivado: {} (ID: {})", hotel.nombre, id);
        
        if let Err(e) = self.logging_service.log_delete::<Hotel>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Hotel,
            id,
            Some(&hotel),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de hotel: {}", e);
        }
        
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Hotel desactivado",
            &format!("{} ha desactivado el hotel '{}'", username, hotel.nombre),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de hotel desactivado: {}", e);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn restore_hotel(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<HotelResponse, ApplicationError> {
        let restored = self.hotel_repository.restore(id, restored_by).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Hotel {} no encontrado", id)));
        }
        
        let hotel = self.hotel_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id)))?;
        
        info!("Hotel restaurado: {} (ID: {})", hotel.nombre, id);
        
        if let Err(e) = self.logging_service.log_update::<Hotel>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Hotel,
            id,
            None::<&Hotel>,
            Some(&hotel),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de hotel: {}", e);
        }
        
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Hotel restaurado",
            &format!("{} ha restaurado el hotel '{}'", username, hotel.nombre),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de hotel restaurado: {}", e);
        }

        Ok(HotelResponse::from(hotel))
    }

    #[instrument(skip(self))]
    pub async fn hard_delete_hotel(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        let hotel = self.hotel_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id)))?;
        
        if !self.hotel_repository.hard_delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Hotel {} no encontrado", id)));
        }
        
        info!("[HARD_DELETE] Hotel eliminado permanentemente: {} (ID: {})", hotel.nombre, id);
        
        if let Err(e) = self.logging_service.log_delete::<Hotel>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Hotel,
            id,
            Some(&hotel),
            Some("HARD_DELETE - Eliminacion permanente".to_string()),
        ).await {
            warn!("Error al registrar log de hard_delete de hotel: {}", e);
        }
        
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "Hotel ELIMINADO permanentemente",
            &format!("{} ha eliminado permanentemente el hotel '{}' (ID: {})", username, hotel.nombre, id),
            NotificationType::Warning,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de hotel eliminado: {}", e);
        }
        
        Ok(())
    }

    /// Obtener el hotel del usuario actual (para rol Hoteles)
    #[instrument(skip(self))]
    pub async fn get_mi_hotel(
        &self,
        user_role: &UserRole,
        id_entidad: Option<i32>,
        username: &str,
    ) -> Result<HotelResponse, ApplicationError> {
        info!("Buscando hotel para usuario '{}' (id_entidad: {:?}, role: {:?})", 
            username, id_entidad, user_role);
        
        let mut hotel: Option<Hotel> = None;
        
        let is_hotel_user = *user_role == UserRole::Hoteles;
        
        if is_hotel_user {
            if let Some(entity_id) = id_entidad {
                hotel = self.hotel_repository
                    .find_by_id(entity_id)
                    .await?;
                if hotel.is_some() {
                    info!("Hotel encontrado por id_entidad: {}", entity_id);
                }
            }
        }
        
        match hotel {
            Some(h) => Ok(HotelResponse::from(h)),
            None => {
                info!("Usuario '{}' no tiene hotel asociado", username);
                Err(ApplicationError::NotFound("No tienes un hotel asociado".to_string()))
            }
        }
    }

    fn detect_changed_fields(&self, old: &Hotel, request: &UpdateHotelRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.id_cadena.as_ref().map(|c| *c != old.id_cadena).unwrap_or(false) {
            changed.push("id_cadena".to_string());
        }
        if request.nombre.as_ref().map(|n| n != &old.nombre).unwrap_or(false) {
            changed.push("nombre".to_string());
        }
        if request.categoria.as_ref().map(|c| Some(c.clone()) != old.categoria).unwrap_or(false) {
            changed.push("categoria".to_string());
        }
        if request.direccion.as_ref().map(|d| Some(d.clone()) != old.direccion).unwrap_or(false) {
            changed.push("direccion".to_string());
        }
        if request.telefono.as_ref().map(|t| Some(t.clone()) != old.telefono).unwrap_or(false) {
            changed.push("telefono".to_string());
        }
        if request.correo.as_ref().map(|c| Some(c.clone()) != old.correo).unwrap_or(false) {
            changed.push("correo".to_string());
        }
        if request.ciudad.as_ref().map(|c| Some(c.clone()) != old.ciudad).unwrap_or(false) {
            changed.push("ciudad".to_string());
        }
        if request.is_active.as_ref().map(|a| *a != old.is_active).unwrap_or(false) {
            changed.push("is_active".to_string());
        }
        
        changed
    }
}
