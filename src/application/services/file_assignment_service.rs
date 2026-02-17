//! Servicio de asignaciones de recursos a Files
//!
//! Este servicio maneja la lógica de negocio para:
//! - Notificar a las partes interesadas (admin, agencia, proveedor)
//! - Verificar si el file está completamente asignado
//! - Cambiar el status del file de "confirmado" a "asignado" cuando corresponda
//! - Registrar logs de actividad

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::ports::{
    FileRepositoryPort, FileTourRepositoryPort, NotificationServicePort,
    FileVehiculoRepositoryPort, FileGuiaRepositoryPort,
    FileRestauranteRepositoryPort, FileEntradaRepositoryPort,
    ConductorRepositoryPort, GuiaRepositoryPort, UserRepositoryPort,
};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de asignaciones de recursos a files
pub struct FileAssignmentService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    conductor_repository: Arc<dyn ConductorRepositoryPort>,
    guia_repository: Arc<dyn GuiaRepositoryPort>,
    user_repository: Arc<dyn UserRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl FileAssignmentService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
        _file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
        _file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
        _file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
        conductor_repository: Arc<dyn ConductorRepositoryPort>,
        guia_repository: Arc<dyn GuiaRepositoryPort>,
        user_repository: Arc<dyn UserRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
            file_vehiculo_repository,
            conductor_repository,
            guia_repository,
            user_repository,
            logging_service,
            notification_service,
        }
    }

    // =========================================================================
    // VERIFICACIÓN DE ASIGNACIONES COMPLETAS
    // =========================================================================

    /// Verifica si un file tiene todas las asignaciones necesarias completadas
    /// Un file está completamente asignado cuando todos sus file_tours tienen:
    /// - Al menos un vehículo asignado
    /// - Al menos un conductor asignado
    /// - Un guía asignado (opcional pero recomendado)
    #[instrument(skip(self))]
    pub async fn is_file_fully_assigned(&self, file_id: i32) -> Result<bool, ApplicationError> {
        // Obtener todos los file_tours del file
        let file_tours = self.file_tour_repository
            .find_by_file_with_tour(file_id)
            .await?;
        
        if file_tours.is_empty() {
            return Ok(false);
        }
        
        for ft in &file_tours {
            // Verificar vehículos asignados
            let vehiculos = self.file_vehiculo_repository
                .find_by_file_tour(ft.id)
                .await?;
            
            if vehiculos.is_empty() {
                info!("File {} - FileTour {} sin vehículos asignados", file_id, ft.id);
                return Ok(false);
            }
            
            // Verificar que al menos un vehículo tenga conductor
            let has_conductor = vehiculos.iter().any(|v| v.id_conductor.is_some());
            if !has_conductor {
                info!("File {} - FileTour {} sin conductor asignado", file_id, ft.id);
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Actualiza el status del file a "asignado" si está completamente asignado
    #[instrument(skip(self))]
    pub async fn check_and_update_file_status(
        &self,
        file_id: i32,
        updated_by: i32,
    ) -> Result<bool, ApplicationError> {
        let file = self.file_repository
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
        
        // Solo actualizar si está en estado "confirmado"
        if file.status != "confirmado" {
            return Ok(false);
        }
        
        // Verificar si está completamente asignado
        if !self.is_file_fully_assigned(file_id).await? {
            return Ok(false);
        }
        
        // Actualizar el status a "asignado"
        let mut updated_file = file.clone();
        updated_file.status = "asignado".to_string();
        updated_file.updated_by = Some(updated_by);
        updated_file.updated_at = chrono::Utc::now();
        
        self.file_repository.update(&updated_file).await?;
        
        info!("✅ File {} cambió de 'confirmado' a 'asignado'", file_id);
        
        // Log del evento
        if let Err(e) = self.logging_service.log_update::<crate::domain::entities::File>(
            Some(updated_by),
            None,
            EntityType::File,
            file_id,
            Some(&file),
            Some(&updated_file),
            Some(vec!["status".to_string()]),
            None, // IP no aplica en operación de servicio
        ).await {
            warn!("Error al registrar log de cambio de status: {}", e);
        }
        
        // Notificar a admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "📦 File Completamente Asignado",
            &format!(
                "El file #{} ({}) ha sido completamente asignado y está listo para operar.",
                file_id,
                file.file_code.clone().unwrap_or_else(|| format!("F-{}", file_id))
            ),
            NotificationType::Success,
            NotificationCategory::System,
            NotificationPriority::Normal,
            Some(updated_by),
        ).await {
            warn!("Error al notificar cambio de status: {}", e);
        }
        
        Ok(true)
    }

    // =========================================================================
    // NOTIFICACIONES A PROVEEDORES
    // =========================================================================

    /// Notifica a los usuarios asociados a una entidad (transporte, restaurante, guía)
    #[instrument(skip(self))]
    pub async fn notify_entity_users(
        &self,
        entity_type: &str, // "transporte", "restaurante", "guia"
        entity_id: i32,
        title: &str,
        message: &str,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // Buscar usuarios con id_entidad correspondiente al entity_id y rol apropiado
        let role = match entity_type {
            "transporte" | "conductor" => "transportes",
            "restaurante" => "restaurantes",
            "guia" => "guias",
            _ => return Ok(()), // Ignorar tipos desconocidos
        };
        
        // Buscar usuarios de esta entidad
        let users = self.user_repository
            .find_by_role_and_entity(role, entity_id)
            .await?;
        
        if users.is_empty() {
            info!("No hay usuarios asociados a {} ID {}", entity_type, entity_id);
            return Ok(());
        }
        
        // Notificar a cada usuario de la entidad
        for user in users {
            if let Err(e) = self.notification_service.notify_user(
                user.id,
                title,
                message,
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::High,
                created_by,
            ).await {
                warn!("Error al notificar usuario {} de {}: {}", user.id, entity_type, e);
            }
        }
        
        info!("Notificado a usuarios de {} ID {}", entity_type, entity_id);
        Ok(())
    }

    /// Notifica al conductor asignado sobre su nueva asignación
    #[instrument(skip(self))]
    pub async fn notify_conductor_assignment(
        &self,
        conductor_id: i32,
        file_code: &str,
        tour_name: &str,
        fecha: &str,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // Obtener el conductor para obtener id_persona
        let conductor = self.conductor_repository
            .find_by_id(conductor_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", conductor_id)))?;
        
        // Obtener el transporte asociado al conductor para notificar a la empresa
        if let Some(id_transporte) = conductor.id_transporte {
            self.notify_entity_users(
                "transporte",
                id_transporte,
                "🚐 Nueva asignación de conductor",
                &format!(
                    "Su conductor ha sido asignado al file {} - Tour: {} ({})",
                    file_code, tour_name, fecha
                ),
                created_by,
            ).await?;
        }
        
        Ok(())
    }

    /// Notifica al guía asignado sobre su nueva asignación
    #[instrument(skip(self))]
    pub async fn notify_guia_assignment(
        &self,
        guia_id: i32,
        file_code: &str,
        tour_name: &str,
        fecha: &str,
        rol: &str,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        // Obtener el guía
        let guia = self.guia_repository
            .find_by_id(guia_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", guia_id)))?;
        
        // Buscar usuario asociado a este guía (por id_persona y rol guias)
        let users = self.user_repository
            .find_by_persona_id(guia.id_persona)
            .await?;
        
        for user in users {
            if user.role == UserRole::Guias {
                if let Err(e) = self.notification_service.notify_user(
                    user.id,
                    "🎯 Nueva asignación de guía",
                    &format!(
                        "Has sido asignado como {} al file {} - Tour: {} ({})",
                        rol, file_code, tour_name, fecha
                    ),
                    NotificationType::Info,
                    NotificationCategory::Crud,
                    NotificationPriority::High,
                    created_by,
                ).await {
                    warn!("Error al notificar guía {}: {}", user.id, e);
                }
            }
        }
        
        Ok(())
    }

    /// Notifica al restaurante asignado sobre su nueva asignación
    #[instrument(skip(self))]
    pub async fn notify_restaurante_assignment(
        &self,
        restaurante_id: i32,
        file_code: &str,
        tour_name: &str,
        fecha: &str,
        servicio: &str,
        created_by: Option<i32>,
    ) -> Result<(), ApplicationError> {
        self.notify_entity_users(
            "restaurante",
            restaurante_id,
            "🍽️ Nueva reserva de servicio",
            &format!(
                "Se ha reservado {} para el file {} - Tour: {} ({})",
                servicio, file_code, tour_name, fecha
            ),
            created_by,
        ).await
    }
}
