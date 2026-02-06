//! Agencia Service - Lógica de negocio para agencias
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con agencias:
//! - Creación de agencias (con validaciones de unicidad de RUC)
//! - Actualización de agencias
//! - Desactivación/Restauración
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, debug, instrument};

use crate::application::dtos::{
    CreateAgenciaRequest, UpdateAgenciaRequest, AgenciaResponse, AgenciaListItemDto,
};
use crate::application::ports::{
    AgenciaRepositoryPort, NotificationServicePort, CachePort, entity_names,
};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Agencia, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de agencias - contiene la lógica de negocio
pub struct AgenciaService {
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
    cache: Arc<dyn CachePort>,
}

impl AgenciaService {
    pub fn new(
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
        cache: Arc<dyn CachePort>,
    ) -> Self {
        Self {
            agencia_repository,
            logging_service,
            notification_service,
            cache,
        }
    }

    /// Generar clave de caché para listado
    fn list_cache_key(&self, page: i64, page_size: i64) -> String {
        format!("list:{}:{}", page, page_size)
    }

    /// Listar agencias con paginación (con caché)
    #[instrument(skip(self))]
    pub async fn list_agencias(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<AgenciaListItemDto>, i64), ApplicationError> {
        let cache_key = self.list_cache_key(page, page_size);
        
        // Intentar obtener del caché
        if let Some(cached) = self.cache.get_list(entity_names::AGENCIAS, &cache_key).await {
            debug!("Cache HIT para agencias list: {}", cache_key);
            if let Ok(response) = serde_json::from_str::<(Vec<AgenciaListItemDto>, i64)>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para agencias list: {}", cache_key);
        
        let offset = (page - 1) * page_size;
        let (items, total) = self.agencia_repository
            .list_with_encargado(page_size, offset)
            .await?;
        
        info!("Listadas {} agencias (página {}, total: {})", items.len(), page, total);
        
        let response = (items, total);
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::AGENCIAS, &cache_key, serialized).await;
        }
        
        Ok(response)
    }

    /// Obtener agencia por ID (con caché)
    #[instrument(skip(self))]
    pub async fn get_agencia(&self, id: i32) -> Result<AgenciaResponse, ApplicationError> {
        // Intentar obtener del caché
        if let Some(cached) = self.cache.get_detail(entity_names::AGENCIAS, id).await {
            debug!("Cache HIT para agencia: {}", id);
            if let Ok(response) = serde_json::from_str::<AgenciaResponse>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para agencia: {}", id);
        
        let agencia = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        let response = AgenciaResponse::from(agencia.clone());
        info!("Agencia encontrada: {} (ID: {})", agencia.nombre, id);
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_detail(entity_names::AGENCIAS, id, serialized).await;
        }
        
        Ok(response)
    }

    /// Obtener agencia por RUC
    #[instrument(skip(self))]
    pub async fn get_agencia_by_ruc(&self, ruc: &str) -> Result<AgenciaResponse, ApplicationError> {
        let agencia = self.agencia_repository
            .find_by_ruc(ruc)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia con RUC {} no encontrada", ruc)))?;
        
        info!("Agencia encontrada por RUC: {} ({})", agencia.nombre, ruc);
        Ok(AgenciaResponse::from(agencia))
    }

    /// Crear una nueva agencia
    /// 
    /// # Validaciones de negocio:
    /// - RUC debe ser único
    #[instrument(skip(self, request))]
    pub async fn create_agencia(
        &self,
        request: CreateAgenciaRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<AgenciaResponse, ApplicationError> {
        // Validación de negocio: RUC único
        if self.agencia_repository.exists_by_ruc(&request.ruc).await? {
            return Err(ApplicationError::Conflict(
                format!("Ya existe una agencia con RUC {}", request.ruc)
            ));
        }
        
        // Crear entidad de dominio
        let agencia = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.agencia_repository.create(&agencia).await?;
        info!("Agencia creada: {} (ID: {})", created.nombre, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Agencia>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Agencia,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de agencia: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nueva agencia creada",
            &format!("{} ha creado la agencia '{}' (RUC: {})", username, created.nombre, created.ruc),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de agencia creada: {}", e);
        }
        
        // Invalidar caché de agencias
        self.cache.invalidate_entity(entity_names::AGENCIAS).await;
        
        Ok(AgenciaResponse::from(created))
    }

    /// Actualizar una agencia existente
    /// 
    /// # Validaciones de negocio:
    /// - Agencia debe existir
    /// - Si se cambia el RUC, debe ser único
    #[instrument(skip(self, request))]
    pub async fn update_agencia(
        &self,
        id: i32,
        request: UpdateAgenciaRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<AgenciaResponse, ApplicationError> {
        // Verificar que existe
        let old_agencia = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        // Si se está cambiando el RUC, verificar unicidad
        if let Some(ref new_ruc) = request.ruc {
            if new_ruc != &old_agencia.ruc {
                if self.agencia_repository.exists_by_ruc(new_ruc).await? {
                    return Err(ApplicationError::Conflict(
                        format!("Ya existe una agencia con RUC {}", new_ruc)
                    ));
                }
            }
        }
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_agencia, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_agencia.clone(), Some(updated_by));
        
        // Persistir
        let result = self.agencia_repository.update(&updated_entity).await?;
        info!("✏️ Agencia actualizada: {} (ID: {})", result.nombre, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Agencia>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Agencia,
            id,
            Some(&old_agencia),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de agencia: {}", e);
        }
        
        // Notificación a admins
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Agencia actualizada",
            &format!("{} ha actualizado la agencia '{}'", username, result.nombre),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de agencia actualizada: {}", e);
        }
        
        // Invalidar caché de la agencia específica
        self.cache.invalidate_detail(entity_names::AGENCIAS, id).await;
        
        Ok(AgenciaResponse::from(result))
    }

    /// Desactivar una agencia (soft delete)
    #[instrument(skip(self))]
    pub async fn delete_agencia(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener agencia antes de desactivar
        let agencia = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        // Ejecutar soft delete
        let deleted = self.agencia_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Agencia {} no encontrada", id)));
        }
        
        info!("[DELETE] Agencia desactivada: {} (ID: {})", agencia.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Agencia>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Agencia,
            id,
            Some(&agencia),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de agencia: {}", e);
        }
        
        // Notificación a admins
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Agencia desactivada",
            &format!("{} ha desactivado la agencia '{}'", username, agencia.nombre),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de agencia desactivada: {}", e);
        }
        
        // Invalidar caché de la agencia
        self.cache.invalidate_detail(entity_names::AGENCIAS, id).await;
        
        Ok(())
    }

    /// Restaurar una agencia desactivada
    #[instrument(skip(self))]
    pub async fn restore_agencia(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<AgenciaResponse, ApplicationError> {
        // Ejecutar restauración
        let restored = self.agencia_repository.restore(id, restored_by).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Agencia {} no encontrada", id)));
        }
        
        // Obtener agencia restaurada
        let agencia = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        info!("♻️ Agencia restaurada: {} (ID: {})", agencia.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Agencia>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Agencia,
            id,
            None::<&Agencia>,
            Some(&agencia),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de agencia: {}", e);
        }
        
        // Notificación a admins
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Agencia restaurada",
            &format!("{} ha restaurado la agencia '{}'", username, agencia.nombre),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de agencia restaurada: {}", e);
        }
        
        // Invalidar caché de la agencia
        self.cache.invalidate_detail(entity_names::AGENCIAS, id).await;
        
        Ok(AgenciaResponse::from(agencia))
    }

    /// Eliminación permanente de una agencia (HARD DELETE)
    #[instrument(skip(self))]
    pub async fn hard_delete_agencia(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener agencia antes de eliminar
        let agencia = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        // Verificar que no tenga files activos asociados
        // TODO: agregar verificación cuando se implemente la relación
        
        // Ejecutar hard delete
        if !self.agencia_repository.hard_delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Agencia {} no encontrada", id)));
        }
        
        info!("[HARD_DELETE] Agencia eliminada permanentemente: {} (ID: {})", agencia.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Agencia>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Agencia,
            id,
            Some(&agencia),
            Some("HARD_DELETE - Eliminacion permanente".to_string()),
        ).await {
            warn!("Error al registrar log de hard_delete de agencia: {}", e);
        }
        
        // Notificación a admins (crítica)
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "Agencia ELIMINADA permanentemente",
            &format!("{} ha eliminado permanentemente la agencia '{}' (ID: {})", username, agencia.nombre, id),
            NotificationType::Warning,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de agencia eliminada: {}", e);
        }
        
        // Invalidar caché de agencias
        self.cache.invalidate_entity(entity_names::AGENCIAS).await;
        
        Ok(())
    }

    /// Obtener la agencia del usuario actual
    #[instrument(skip(self))]
    pub async fn get_mi_agencia(
        &self,
        user_role: &UserRole,
        id_entidad: Option<i32>,
        id_persona: Option<i32>,
        username: &str,
    ) -> Result<AgenciaResponse, ApplicationError> {
        info!("Buscando agencia para usuario '{}' (id_persona: {:?}, id_entidad: {:?}, role: {:?})", 
            username, id_persona, id_entidad, user_role);
        
        let mut agencia: Option<Agencia> = None;
        
        // Verificar si el usuario tiene rol de agencia (incluye agencias y agencias_contador)
        let is_agencia_user = *user_role == UserRole::Agencias || *user_role == UserRole::AgenciasContador;
        
        // Primero intentar por id_entidad si el usuario está relacionado con una agencia
        if is_agencia_user {
            if let Some(entity_id) = id_entidad {
                agencia = self.agencia_repository
                    .find_by_id(entity_id)
                    .await?;
                if agencia.is_some() {
                    info!("Agencia encontrada por id_entidad: {}", entity_id);
                }
            }
        }
        
        // Si no se encontró, buscar por encargado (id_persona)
        if agencia.is_none() {
            if let Some(persona_id) = id_persona {
                agencia = self.agencia_repository
                    .find_by_encargado(persona_id)
                    .await?;
                if agencia.is_some() {
                    info!("Agencia encontrada por encargado (persona_id: {})", persona_id);
                }
            }
        }
        
        match agencia {
            Some(a) => Ok(AgenciaResponse::from(a)),
            None => {
                info!("ℹ️ Usuario '{}' no tiene agencia asociada", username);
                Err(ApplicationError::NotFound("No tienes una agencia asociada".to_string()))
            }
        }
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &Agencia, request: &UpdateAgenciaRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.nombre.as_ref().map(|n| n != &old.nombre).unwrap_or(false) {
            changed.push("nombre".to_string());
        }
        if request.ruc.as_ref().map(|r| r != &old.ruc).unwrap_or(false) {
            changed.push("ruc".to_string());
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
        if request.encargado.is_some() && request.encargado != old.encargado {
            changed.push("encargado".to_string());
        }
        if request.paleta_colores.is_some() {
            changed.push("paleta_colores".to_string());
        }
        if request.media.is_some() {
            changed.push("media".to_string());
        }
        if request.is_active.as_ref().map(|a| *a != old.is_active).unwrap_or(false) {
            changed.push("is_active".to_string());
        }
        
        changed
    }
}
