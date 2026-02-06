//! Restaurante Service - Lógica de negocio para restaurantes
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con restaurantes:
//! - Creación de restaurantes
//! - Actualización de restaurantes  
//! - Desactivación (soft delete)
//! - Restauración
//! - Búsqueda por tipo de atención y capacidad
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use tracing::{info, warn, debug, instrument};

use crate::application::dtos::{
    CreateRestauranteRequest, UpdateRestauranteRequest, RestauranteResponse, RestauranteListItemDto,
};
use crate::application::ports::{RestauranteRepositoryPort, PaginationOptions, NotificationServicePort, CachePort, entity_names};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    Restaurante, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Servicio de restaurantes - contiene la lógica de negocio
pub struct RestauranteService {
    restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
    cache: Arc<dyn CachePort>,
}

impl RestauranteService {
    pub fn new(
        restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
        cache: Arc<dyn CachePort>,
    ) -> Self {
        Self {
            restaurante_repository,
            logging_service,
            notification_service,
            cache,
        }
    }

    /// Generar clave de caché para listado
    fn list_cache_key(&self, limit: i64, offset: i64) -> String {
        format!("list:{}:{}", limit, offset)
    }

    // ===== Métodos de consulta =====

    /// Listar restaurantes con paginación (incluye nombre del encargado, con caché)
    #[instrument(skip(self))]
    pub async fn list_restaurantes(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<RestauranteListItemDto>, i64), ApplicationError> {
        let cache_key = self.list_cache_key(limit, offset);
        
        // Intentar obtener del caché
        if let Some(cached) = self.cache.get_list(entity_names::RESTAURANTES, &cache_key).await {
            debug!("Cache HIT para restaurantes list: {}", cache_key);
            if let Ok(response) = serde_json::from_str::<(Vec<RestauranteListItemDto>, i64)>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para restaurantes list: {}", cache_key);
        
        let (items, total) = self.restaurante_repository.list_with_encargado(limit, offset).await?;
        info!("Listados {} restaurantes (offset: {}, total: {})", items.len(), offset, total);
        
        let response = (items, total);
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::RESTAURANTES, &cache_key, serialized).await;
        }
        
        Ok(response)
    }

    /// Obtener restaurante por ID (con caché)
    #[instrument(skip(self))]
    pub async fn get_restaurante(&self, id: i32) -> Result<RestauranteResponse, ApplicationError> {
        // Intentar obtener del caché
        if let Some(cached) = self.cache.get_detail(entity_names::RESTAURANTES, id).await {
            debug!("Cache HIT para restaurante: {}", id);
            if let Ok(response) = serde_json::from_str::<RestauranteResponse>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para restaurante: {}", id);
        
        let restaurante = self.restaurante_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
        
        let response = RestauranteResponse::from(restaurante.clone());
        info!("Restaurante encontrado: {} (ID: {})", restaurante.nombre, id);
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_detail(entity_names::RESTAURANTES, id, serialized).await;
        }
        
        Ok(response)
    }

    /// Buscar restaurantes por tipo de atención
    #[instrument(skip(self))]
    pub async fn search_by_tipo_atencion(&self, tipo: &str) -> Result<Vec<RestauranteResponse>, ApplicationError> {
        let restaurantes = self.restaurante_repository.find_by_tipo_atencion(tipo).await?;
        info!("🔎 Encontrados {} restaurantes con tipo de atención: {}", restaurantes.len(), tipo);
        Ok(restaurantes.into_iter().map(RestauranteResponse::from).collect())
    }

    /// Buscar restaurantes por capacidad mínima
    #[instrument(skip(self))]
    pub async fn search_by_min_capacity(&self, min_capacity: i32) -> Result<Vec<RestauranteResponse>, ApplicationError> {
        let restaurantes = self.restaurante_repository.find_by_min_capacity(min_capacity).await?;
        info!("🔎 Encontrados {} restaurantes con capacidad >= {}", restaurantes.len(), min_capacity);
        Ok(restaurantes.into_iter().map(RestauranteResponse::from).collect())
    }

    /// Listar restaurantes con paginación simple (sin encargado)
    #[instrument(skip(self))]
    pub async fn list_simple(
        &self,
        options: PaginationOptions,
    ) -> Result<Vec<RestauranteResponse>, ApplicationError> {
        let result = self.restaurante_repository.list_paginated(options).await?;
        Ok(result.data.into_iter().map(RestauranteResponse::from).collect())
    }

    // ===== Métodos de mutación =====

    /// Crear un nuevo restaurante
    #[instrument(skip(self, request))]
    pub async fn create_restaurante(
        &self,
        request: CreateRestauranteRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<RestauranteResponse, ApplicationError> {
        // Validación adicional de negocio si es necesaria
        if request.nombre.trim().is_empty() {
            return Err(ApplicationError::Validation("El nombre del restaurante es requerido".into()));
        }
        
        // Crear entidad de dominio
        let restaurante = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.restaurante_repository.create(&restaurante).await?;
        info!("Restaurante creado: {} (ID: {})", created.nombre, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<Restaurante>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::Restaurante,
            created.id,
            &created.nombre,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de restaurante: {}", e);
        }
        
        // Notificación a admins
        let username = created_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo restaurante creado",
            &format!("{} ha creado el restaurante '{}' en {}", username, created.nombre, created.direccion),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de restaurante creado: {}", e);
        }
        
        // Invalidar caché de restaurantes
        self.cache.invalidate_entity(entity_names::RESTAURANTES).await;
        
        Ok(RestauranteResponse::from(created))
    }

    /// Actualizar un restaurante existente
    #[instrument(skip(self, request))]
    pub async fn update_restaurante(
        &self,
        id: i32,
        request: UpdateRestauranteRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<RestauranteResponse, ApplicationError> {
        // Verificar que existe
        let old_restaurante = self.restaurante_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_restaurante, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_restaurante.clone(), Some(updated_by));
        
        // Persistir
        let result = self.restaurante_repository.update(&updated_entity).await?;
        info!("✏️ Restaurante actualizado: {} (ID: {})", result.nombre, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Restaurante>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::Restaurante,
            id,
            Some(&old_restaurante),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de restaurante: {}", e);
        }
        
        // Determinar prioridad de notificación según campos cambiados
        let priority = if changed_fields.iter().any(|f| f == "nombre" || f == "tipo_atencion" || f == "capacidad") {
            NotificationPriority::Normal
        } else {
            NotificationPriority::Low
        };
        
        // Notificación a admins
        let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
        let fields_str = if changed_fields.is_empty() {
            "sin cambios específicos".to_string()
        } else {
            changed_fields.join(", ")
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Restaurante actualizado",
            &format!("{} ha actualizado el restaurante '{}'. Campos: {}", username, result.nombre, fields_str),
            NotificationType::Info,
            NotificationCategory::Crud,
            priority,
            Some(updated_by),
        ).await {
            warn!("Error al enviar notificación de restaurante actualizado: {}", e);
        }
        
        // Invalidar caché del restaurante específico
        self.cache.invalidate_detail(entity_names::RESTAURANTES, id).await;
        
        Ok(RestauranteResponse::from(result))
    }

    /// Desactivar (soft delete) un restaurante
    #[instrument(skip(self))]
    pub async fn deactivate_restaurante(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let restaurante = self.restaurante_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
        
        if !restaurante.is_active {
            return Err(ApplicationError::Validation("El restaurante ya está desactivado".into()));
        }
        
        // Desactivar
        let mut updated = restaurante.clone();
        updated.is_active = false;
        updated.updated_by = Some(deleted_by);
        updated.updated_at = chrono::Utc::now();
        
        self.restaurante_repository.update(&updated).await?;
        info!("[DELETE] Restaurante desactivado: {} (ID: {})", restaurante.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Restaurante>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Restaurante,
            id,
            Some(&restaurante),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de restaurante: {}", e);
        }
        
        // Notificación a admins - Warning porque es una eliminación
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Restaurante desactivado",
            &format!("{} ha desactivado el restaurante '{}' ({})", username, restaurante.nombre, restaurante.direccion),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de restaurante desactivado: {}", e);
        }
        
        // Invalidar caché del restaurante
        self.cache.invalidate_detail(entity_names::RESTAURANTES, id).await;
        
        Ok(())
    }

    /// Eliminación permanente de un restaurante (hard delete) - Solo SuperAdmin
    #[instrument(skip(self))]
    pub async fn hard_delete_restaurante(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let restaurante = self.restaurante_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
        
        // Eliminar permanentemente
        if !self.restaurante_repository.hard_delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)));
        }
        info!("[DELETE] Restaurante ELIMINADO PERMANENTEMENTE: {} (ID: {})", restaurante.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Restaurante>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::Restaurante,
            id,
            Some(&restaurante),
            Some("HARD_DELETE - Eliminación permanente".to_string()),
        ).await {
            warn!("Error al registrar log de eliminación permanente de restaurante: {}", e);
        }
        
        // Notificación a SuperAdmins
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "Restaurante eliminado permanentemente",
            &format!("{} ha eliminado permanentemente el restaurante '{}' (ID: {})", username, restaurante.nombre, id),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de restaurante eliminado permanentemente: {}", e);
        }
        
        // Invalidar caché de restaurantes
        self.cache.invalidate_entity(entity_names::RESTAURANTES).await;
        
        Ok(())
    }

    /// Restaurar un restaurante desactivado
    #[instrument(skip(self))]
    pub async fn restore_restaurante(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Verificar que existe
        let restaurante = self.restaurante_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
        
        if restaurante.is_active {
            return Err(ApplicationError::Validation("El restaurante ya está activo".into()));
        }
        
        // Restaurar
        let mut updated = restaurante.clone();
        updated.is_active = true;
        updated.updated_by = Some(restored_by);
        updated.updated_at = chrono::Utc::now();
        
        let restored = self.restaurante_repository.update(&updated).await?;
        info!("♻️ Restaurante restaurado: {} (ID: {})", restaurante.nombre, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Restaurante>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::Restaurante,
            id,
            Some(&restaurante),
            Some(&restored),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de restaurante: {}", e);
        }
        
        // Notificación a admins - Success porque se recupera
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Restaurante restaurado",
            &format!("{} ha restaurado el restaurante '{}' ({})", username, restaurante.nombre, restaurante.direccion),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de restaurante restaurado: {}", e);
        }
        
        // Invalidar caché del restaurante
        self.cache.invalidate_detail(entity_names::RESTAURANTES, id).await;
        
        Ok(())
    }

    // ===== Métodos auxiliares privados =====

    /// Detectar campos que fueron modificados
    fn detect_changed_fields(&self, old: &Restaurante, request: &UpdateRestauranteRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if let Some(ref nombre) = request.nombre {
            if nombre != &old.nombre {
                changed.push("nombre".to_string());
            }
        }
        if let Some(ref direccion) = request.direccion {
            if direccion != &old.direccion {
                changed.push("direccion".to_string());
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
        if request.tipo_atencion.is_some() {
            // Comparar tipo_atencion es más complejo porque es JSON
            changed.push("tipo_atencion".to_string());
        }
        if let Some(precio) = request.precio_promedio {
            let old_precio = old.precio_promedio.as_ref()
                .and_then(|p| p.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            if (precio - old_precio).abs() > 0.001 {
                changed.push("precio_promedio".to_string());
            }
        }
        if let Some(capacidad) = request.capacidad {
            if Some(capacidad) != old.capacidad {
                changed.push("capacidad".to_string());
            }
        }
        if request.horario.is_some() {
            changed.push("horario".to_string());
        }
        if request.encargado != old.encargado {
            changed.push("encargado".to_string());
        }
        if let Some(is_active) = request.is_active {
            if is_active != old.is_active {
                changed.push("is_active".to_string());
            }
        }
        
        changed
    }
}
