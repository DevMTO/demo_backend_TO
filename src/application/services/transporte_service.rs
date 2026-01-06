//! Transporte Service - Lógica de negocio para empresas de transporte
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con transportes:
//! - Creación y administración de empresas de transporte
//! - Validación de RUC único
//! - Operaciones para usuarios con rol de transporte
//! - Soft delete y restauración
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateTransporteRequest, UpdateTransporteRequest, UpdateTransporteInterfazRequest,
    TransporteResponse, TransporteListItemDto,
};
use crate::application::ports::{TransporteRepositoryPort, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Transporte, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de transportes - contiene la lógica de negocio
pub struct TransporteService {
    transporte_repository: Arc<dyn TransporteRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl TransporteService {
    pub fn new(
        transporte_repository: Arc<dyn TransporteRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            transporte_repository,
            logging_service,
            notification_service,
        }
    }

    // ===== Métodos de consulta =====

    /// Listar transportes con paginación (incluye nombre del encargado)
    #[instrument(skip(self))]
    pub async fn list_transportes(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TransporteListItemDto>, i64), ApplicationError> {
        let (items, total) = self.transporte_repository.list_with_encargado(limit, offset).await?;
        info!("📋 Listados {} transportes (offset: {}, total: {})", items.len(), offset, total);
        Ok((items, total))
    }

    /// Obtener transporte por ID
    #[instrument(skip(self))]
    pub async fn get_transporte(&self, id: i32) -> Result<TransporteResponse, ApplicationError> {
        let transporte = self.transporte_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
        
        info!("🔍 Transporte encontrado: {} (ID: {})", transporte.nombre, id);
        Ok(TransporteResponse::from(transporte))
    }

    /// Buscar transporte por encargado (id_persona)
    #[instrument(skip(self))]
    pub async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<TransporteResponse>, ApplicationError> {
        let transporte = self.transporte_repository.find_by_encargado(persona_id).await?;
        Ok(transporte.map(TransporteResponse::from))
    }

    /// Listar transportes con vehículos disponibles
    #[instrument(skip(self))]
    pub async fn list_with_available_vehicles(&self) -> Result<Vec<TransporteResponse>, ApplicationError> {
        let transportes = self.transporte_repository.find_with_available_vehicles().await?;
        info!("🚐 Encontrados {} transportes con vehículos disponibles", transportes.len());
        Ok(transportes.into_iter().map(TransporteResponse::from).collect())
    }

    // ===== Métodos de mutación =====

    /// Crear un nuevo transporte
    #[instrument(skip(self, request))]
    pub async fn create_transporte(
        &self,
        request: CreateTransporteRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<TransporteResponse, ApplicationError> {
        // Validar RUC único
        if self.transporte_repository.exists_by_ruc(&request.ruc).await? {
            return Err(ApplicationError::Conflict(format!("RUC {} ya existe", request.ruc)));
        }
        
        // Crear entidad de dominio
        let transporte = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.transporte_repository.create(&transporte).await?;
        info!("✅ Transporte creado: {} (ID: {})", created.nombre, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Transporte>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Transporte,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de creación de transporte: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo transporte creado",
            &format!("{} ha creado el transporte '{}' (RUC: {})", username, created.nombre, created.ruc),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de transporte creado: {}", e);
        }
        
        Ok(TransporteResponse::from(created))
    }

    /// Actualizar un transporte existente (por admin)
    #[instrument(skip(self, request))]
    pub async fn update_transporte(
        &self,
        id: i32,
        request: UpdateTransporteRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<TransporteResponse, ApplicationError> {
        // Verificar que existe
        let old_transporte = self.transporte_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_transporte, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_transporte.clone(), Some(updated_by));
        
        // Persistir
        let result = self.transporte_repository.update(&updated_entity).await?;
        info!("✏️ Transporte actualizado: {} (ID: {})", result.nombre, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Transporte>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Transporte,
            id,
            Some(&old_transporte),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("⚠️ Error al registrar log de actualización de transporte: {}", e);
        }
        
        // Notificación a admins
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        let fields_str = if changed_fields.is_empty() {
            "sin cambios específicos".to_string()
        } else {
            changed_fields.join(", ")
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Transporte actualizado",
            &format!("{} ha actualizado el transporte '{}'. Campos: {}", username, result.nombre, fields_str),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(updated_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de transporte actualizado: {}", e);
        }
        
        Ok(TransporteResponse::from(result))
    }

    /// Actualizar mi propio transporte (usuario con rol Transportes)
    /// Solo permite ciertos campos: direccion, telefono, correo, paleta_colores, media
    #[instrument(skip(self, request))]
    pub async fn update_my_transporte(
        &self,
        transporte_id: i32,
        request: UpdateTransporteRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<TransporteResponse, ApplicationError> {
        let old_transporte = self.transporte_repository
            .find_by_id(transporte_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("No tienes un transporte asociado".to_string()))?;
        
        // Crear un request limitado: solo permitir ciertos campos
        let limited_request = UpdateTransporteRequest {
            nombre: None, // No puede cambiar nombre
            ruc: None, // No puede cambiar RUC
            direccion: request.direccion,
            telefono: request.telefono,
            correo: request.correo,
            encargado: None, // No puede cambiar encargado
            paleta_colores: request.paleta_colores,
            media: request.media, // Puede actualizar media (logo, etc.)
            is_active: None, // No puede cambiar estado
        };
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_transporte, &limited_request);
        
        // Aplicar cambios
        let updated = limited_request.apply_to(old_transporte.clone(), Some(updated_by));
        let result = self.transporte_repository.update(&updated).await?;
        info!("✏️ Transporte actualizado por su usuario: {} (ID: {})", result.nombre, transporte_id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Transporte>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Transporte,
            transporte_id,
            Some(&old_transporte),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields) },
            None,
        ).await {
            warn!("⚠️ Error al registrar log de actualización de transporte: {}", e);
        }
        
        Ok(TransporteResponse::from(result))
    }

    /// Actualizar solo la interfaz del transporte (paleta_colores y media)
    #[instrument(skip(self, request))]
    pub async fn update_interface(
        &self,
        transporte_id: i32,
        request: UpdateTransporteInterfazRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<TransporteResponse, ApplicationError> {
        let old_transporte = self.transporte_repository
            .find_by_id(transporte_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("No tienes un transporte asociado".to_string()))?;
        
        // Aplicar cambios solo de interfaz
        let updated = request.apply_to(old_transporte.clone(), Some(updated_by));
        let result = self.transporte_repository.update(&updated).await?;
        info!("🎨 Interfaz de transporte actualizada: {} (ID: {})", result.nombre, transporte_id);
        
        // Logging
        if let Err(e) = self.logging_service.log_update::<Transporte>(
            Some(updated_by),
            updated_by_username,
            EntityType::Transporte,
            transporte_id,
            Some(&old_transporte),
            Some(&result),
            Some(vec!["paleta_colores".to_string(), "media".to_string()]),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de actualización de interfaz: {}", e);
        }
        
        Ok(TransporteResponse::from(result))
    }

    /// Desactivar (soft delete) un transporte
    #[instrument(skip(self))]
    pub async fn deactivate_transporte(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let transporte = self.transporte_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
        
        // Soft delete via repository
        if !self.transporte_repository.soft_delete(id, deleted_by).await? {
            return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
        }
        info!("🗑️ Transporte desactivado: {} (ID: {})", transporte.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Transporte>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Transporte,
            id,
            Some(&transporte),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de desactivación de transporte: {}", e);
        }
        
        // Notificación a admins - Warning porque es una eliminación
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Transporte desactivado",
            &format!("{} ha desactivado el transporte '{}' (RUC: {})", username, transporte.nombre, transporte.ruc),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de transporte desactivado: {}", e);
        }
        
        Ok(())
    }

    /// Restaurar un transporte desactivado
    #[instrument(skip(self))]
    pub async fn restore_transporte(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Restore via repository
        if !self.transporte_repository.restore(id, restored_by).await? {
            return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
        }
        
        // Obtener transporte restaurado
        let transporte = self.transporte_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
        
        info!("♻️ Transporte restaurado: {} (ID: {})", transporte.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Transporte>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Transporte,
            id,
            None,
            Some(&transporte),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("⚠️ Error al registrar log de restauración de transporte: {}", e);
        }
        
        // Notificación a admins - Success porque se recupera
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Transporte restaurado",
            &format!("{} ha restaurado el transporte '{}' (RUC: {})", username, transporte.nombre, transporte.ruc),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("⚠️ Error al enviar notificación de transporte restaurado: {}", e);
        }
        
        Ok(())
    }

    // ===== Métodos auxiliares privados =====

    /// Detectar campos que fueron modificados
    fn detect_changed_fields(&self, old: &Transporte, request: &UpdateTransporteRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if let Some(ref nombre) = request.nombre {
            if nombre != &old.nombre {
                changed.push("nombre".to_string());
            }
        }
        if let Some(ref ruc) = request.ruc {
            if ruc != &old.ruc {
                changed.push("ruc".to_string());
            }
        }
        if let Some(ref telefono) = request.telefono {
            if Some(telefono) != old.telefono.as_ref() {
                changed.push("telefono".to_string());
            }
        }
        if let Some(ref correo) = request.correo {
            if Some(correo) != old.correo.as_ref() {
                changed.push("correo".to_string());
            }
        }
        if let Some(ref direccion) = request.direccion {
            if Some(direccion) != old.direccion.as_ref() {
                changed.push("direccion".to_string());
            }
        }
        if request.encargado != old.encargado {
            changed.push("encargado".to_string());
        }
        if request.media.is_some() {
            changed.push("media".to_string());
        }
        if request.paleta_colores.is_some() {
            changed.push("paleta_colores".to_string());
        }
        if let Some(is_active) = request.is_active {
            if is_active != old.is_active {
                changed.push("is_active".to_string());
            }
        }
        
        changed
    }
}
