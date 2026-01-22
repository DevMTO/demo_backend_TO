//! Vehiculo Service - Lógica de negocio para vehículos
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con vehículos:
//! - CRUD de vehículos
//! - Validación de placa única
//! - Filtrado por transporte y disponibilidad
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateVehiculoRequest, UpdateVehiculoRequest, VehiculoResponse, VehiculoListItemDto,
};
use crate::application::ports::{VehiculoRepositoryPort, NotificationServicePort};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Vehiculo, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de vehículos - contiene la lógica de negocio
pub struct VehiculoService {
    vehiculo_repository: Arc<dyn VehiculoRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl VehiculoService {
    pub fn new(
        vehiculo_repository: Arc<dyn VehiculoRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            vehiculo_repository,
            logging_service,
            notification_service,
        }
    }

    // ===== Métodos de consulta =====

    /// Listar vehículos con detalles (nombre del transporte)
    #[instrument(skip(self))]
    pub async fn list_vehiculos(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<VehiculoListItemDto>, i64), ApplicationError> {
        let (items, total) = self.vehiculo_repository.list_with_details(limit, offset).await?;
        info!("Listados {} vehículos (offset: {}, total: {})", items.len(), offset, total);
        Ok((items, total))
    }

    /// Obtener vehículo por ID
    #[instrument(skip(self))]
    pub async fn get_vehiculo(&self, id: i32) -> Result<VehiculoResponse, ApplicationError> {
        let vehiculo = self.vehiculo_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;
        
        info!("Vehículo encontrado: {} (ID: {})", vehiculo.placa, id);
        Ok(VehiculoResponse::from(vehiculo))
    }

    /// Listar vehículos por transporte
    #[instrument(skip(self))]
    pub async fn list_by_transporte(&self, transporte_id: i32) -> Result<Vec<VehiculoListItemDto>, ApplicationError> {
        let vehiculos = self.vehiculo_repository.find_by_transporte_with_details(transporte_id).await?;
        info!("🚐 Encontrados {} vehículos para transporte {}", vehiculos.len(), transporte_id);
        Ok(vehiculos)
    }

    /// Listar vehículos disponibles
    #[instrument(skip(self))]
    pub async fn list_available(&self) -> Result<Vec<VehiculoResponse>, ApplicationError> {
        let vehiculos = self.vehiculo_repository.list_available().await?;
        info!("Encontrados {} vehículos disponibles", vehiculos.len());
        Ok(vehiculos.into_iter().map(VehiculoResponse::from).collect())
    }

    // ===== Métodos de mutación =====

    /// Crear un nuevo vehículo
    #[instrument(skip(self, request))]
    pub async fn create_vehiculo(
        &self,
        request: CreateVehiculoRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<VehiculoResponse, ApplicationError> {
        // Validar placa única
        if self.vehiculo_repository.exists_by_placa(&request.placa).await? {
            return Err(ApplicationError::Conflict(format!("Placa {} ya existe", request.placa)));
        }
        
        // Crear entidad de dominio
        let vehiculo = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.vehiculo_repository.create(&vehiculo).await?;
        info!("Vehículo creado: {} - {} (ID: {})", created.nombre, created.placa, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Vehiculo>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Vehiculo,
            created.id,
            &created.placa,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de vehículo: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo vehículo creado",
            &format!("{} ha creado el vehículo '{}' con placa '{}' (capacidad: {})", 
                     username, created.nombre, created.placa, created.capacidad),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de vehículo creado: {}", e);
        }
        
        Ok(VehiculoResponse::from(created))
    }

    /// Actualizar un vehículo existente
    #[instrument(skip(self, request))]
    pub async fn update_vehiculo(
        &self,
        id: i32,
        request: UpdateVehiculoRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<VehiculoResponse, ApplicationError> {
        // Verificar que existe
        let old_vehiculo = self.vehiculo_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_vehiculo, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_vehiculo.clone(), Some(updated_by));
        
        // Persistir
        let result = self.vehiculo_repository.update(&updated_entity).await?;
        info!("✏️ Vehículo actualizado: {} (ID: {})", result.placa, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Vehiculo>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Vehiculo,
            id,
            Some(&old_vehiculo),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de vehículo: {}", e);
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
            "Vehículo actualizado",
            &format!("{} ha actualizado el vehículo '{}'. Campos: {}", username, result.placa, fields_str),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de vehículo actualizado: {}", e);
        }
        
        Ok(VehiculoResponse::from(result))
    }

    /// Eliminar un vehículo (hard delete)
    #[instrument(skip(self))]
    pub async fn delete_vehiculo(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let vehiculo = self.vehiculo_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;
        
        // Eliminar
        if !self.vehiculo_repository.delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)));
        }
        info!("🗑️ Vehículo eliminado: {} (ID: {})", vehiculo.placa, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Vehiculo>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Vehiculo,
            id,
            Some(&vehiculo),
            None,
        ).await {
            warn!("Error al registrar log de eliminación de vehículo: {}", e);
        }
        
        // Notificación a admins - Warning porque es una eliminación
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Vehículo eliminado",
            &format!("{} ha eliminado el vehículo '{}' con placa '{}'", username, vehiculo.nombre, vehiculo.placa),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de vehículo eliminado: {}", e);
        }
        
        Ok(())
    }

    // ===== Métodos auxiliares privados =====

    /// Detectar campos que fueron modificados
    fn detect_changed_fields(&self, old: &Vehiculo, request: &UpdateVehiculoRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if let Some(id_transporte) = request.id_transporte {
            if id_transporte != old.id_transporte {
                changed.push("id_transporte".to_string());
            }
        }
        if let Some(ref nombre) = request.nombre {
            if nombre != &old.nombre {
                changed.push("nombre".to_string());
            }
        }
        if let Some(ref modelo) = request.modelo {
            if Some(modelo) != old.modelo.as_ref() {
                changed.push("modelo".to_string());
            }
        }
        if let Some(ref placa) = request.placa {
            if placa.to_uppercase() != old.placa {
                changed.push("placa".to_string());
            }
        }
        if let Some(capacidad) = request.capacidad {
            if capacidad != old.capacidad {
                changed.push("capacidad".to_string());
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
