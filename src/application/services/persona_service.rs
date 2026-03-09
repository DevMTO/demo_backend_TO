//! Persona Service - Lógica de negocio para personas
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con personas:
//! - Creación de personas (con validaciones de unicidad de documento)
//! - Actualización de personas
//! - Eliminación
//! - Búsqueda
//! - Logging de actividades

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreatePersonaRequest, UpdatePersonaRequest, PersonaResponse,
};
use crate::application::ports::{PersonaRepositoryPort, PaginationOptions};
use crate::application::services::LoggingService;
use crate::domain::entities::{Persona, EntityType};
use crate::domain::errors::ApplicationError;

/// Servicio de personas - contiene la lógica de negocio
pub struct PersonaService {
    persona_repository: Arc<dyn PersonaRepositoryPort>,
    logging_service: Arc<LoggingService>,
}

impl PersonaService {
    pub fn new(
        persona_repository: Arc<dyn PersonaRepositoryPort>,
        logging_service: Arc<LoggingService>,
    ) -> Self {
        Self {
            persona_repository,
            logging_service,
        }
    }

    /// Listar personas con paginación
    #[instrument(skip(self))]
    pub async fn list_personas(
        &self,
        options: PaginationOptions,
    ) -> Result<(Vec<PersonaResponse>, i64, i64), ApplicationError> {
        let result = self.persona_repository
            .list_paginated(options)
            .await?;
        
        let total = result.total;
        let pages = result.pages();
        let current_page = result.current_page();
        let items: Vec<PersonaResponse> = result.data.into_iter().map(Into::into).collect();
        info!("Listadas {} personas (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener persona por ID
    #[instrument(skip(self))]
    pub async fn get_persona(&self, id: i32) -> Result<PersonaResponse, ApplicationError> {
        let persona = self.persona_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)))?;
        
        info!("Persona encontrada: {} {} (ID: {})", persona.nombre, persona.apellidos, id);
        Ok(PersonaResponse::from(persona))
    }

    /// Crear una nueva persona
    /// 
    /// # Validaciones de negocio:
    /// - Documento debe ser único (tipo_documento + nro_documento)
    #[instrument(skip(self, request))]
    pub async fn create_persona(
        &self,
        request: CreatePersonaRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<PersonaResponse, ApplicationError> {
        // Validación de negocio: documento único
        if self.persona_repository
            .exists_by_documento(&request.tipo_documento, &request.nro_documento)
            .await? 
        {
            return Err(ApplicationError::Conflict(
                format!("Ya existe una persona con {} {}", request.tipo_documento, request.nro_documento)
            ));
        }
        
        // Crear entidad de dominio
        let persona = request.into_entity(Some(created_by));
        
        // Persistir
        let created = self.persona_repository.create(&persona).await?;
        info!("Persona creada: {} {} (ID: {})", created.nombre, created.apellidos, created.id);
        
        // Logging del evento
        let nombre_completo = format!("{} {}", created.nombre, created.apellidos);
        if let Err(e) = self.logging_service.log_create::<Persona>(
            Some(created_by),
            created_by_username,
            EntityType::Persona,
            created.id,
            &nombre_completo,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de persona: {}", e);
        }
        
        Ok(PersonaResponse::from(created))
    }

    /// Actualizar una persona existente
    /// 
    /// # Validaciones de negocio:
    /// - Persona debe existir
    /// - Si se cambia el documento, debe ser único
    #[instrument(skip(self, request))]
    pub async fn update_persona(
        &self,
        id: i32,
        request: UpdatePersonaRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<PersonaResponse, ApplicationError> {
        // Verificar que existe
        let old_persona = self.persona_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)))?;
        
        // Si se está cambiando el documento, verificar unicidad
        let old_tipo_str = old_persona.tipo_documento.to_string();
        let new_tipo = request.tipo_documento.as_ref().unwrap_or(&old_tipo_str);
        let new_nro = request.nro_documento.as_ref().unwrap_or(&old_persona.nro_documento);
        
        if (new_tipo != &old_tipo_str || new_nro != &old_persona.nro_documento)
            && self.persona_repository.exists_by_documento(new_tipo, new_nro).await?
        {
            return Err(ApplicationError::Conflict(
                format!("Ya existe una persona con {} {}", new_tipo, new_nro)
            ));
        }
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_persona, &request);
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_persona.clone(), Some(updated_by));
        
        // Persistir
        let result = self.persona_repository.update(&updated_entity).await?;
        info!("✏️ Persona actualizada: {} {} (ID: {})", result.nombre, result.apellidos, result.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<Persona>(
            Some(updated_by),
            updated_by_username,
            EntityType::Persona,
            id,
            Some(&old_persona),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de persona: {}", e);
        }
        
        Ok(PersonaResponse::from(result))
    }

    /// Eliminar una persona
    #[instrument(skip(self))]
    pub async fn delete_persona(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener persona antes de eliminar
        let persona = self.persona_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)))?;
        
        // Eliminar
        let deleted = self.persona_repository.delete(id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)));
        }
        
        info!("[DELETE] Persona eliminada: {} {} (ID: {})", persona.nombre, persona.apellidos, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<Persona>(
            Some(deleted_by),
            deleted_by_username,
            EntityType::Persona,
            id,
            Some(&persona),
            None,
        ).await {
            warn!("Error al registrar log de eliminación de persona: {}", e);
        }
        
        Ok(())
    }

    /// Eliminación permanente de persona (hard delete) - Solo SuperAdmin
    /// Alias de delete_persona que ya hace eliminación permanente
    #[instrument(skip(self))]
    pub async fn hard_delete_persona(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        self.delete_persona(id, deleted_by, deleted_by_username).await
    }

    /// Buscar personas por texto
    #[instrument(skip(self))]
    pub async fn search_personas(&self, query: &str) -> Result<Vec<PersonaResponse>, ApplicationError> {
        let personas = self.persona_repository
            .search(query)
            .await?;
        
        info!("Búsqueda '{}' encontró {} personas", query, personas.len());
        Ok(personas.into_iter().map(Into::into).collect())
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &Persona, request: &UpdatePersonaRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.nombre.as_ref().map(|n| n != &old.nombre).unwrap_or(false) {
            changed.push("nombre".to_string());
        }
        if request.apellidos.as_ref().map(|a| a != &old.apellidos).unwrap_or(false) {
            changed.push("apellidos".to_string());
        }
        if request.tipo_documento.as_ref().map(|t| t != &old.tipo_documento.to_string()).unwrap_or(false) {
            changed.push("tipo_documento".to_string());
        }
        if request.nro_documento.as_ref().map(|n| n != &old.nro_documento).unwrap_or(false) {
            changed.push("nro_documento".to_string());
        }
        if request.telefono.as_ref().map(|t| Some(t.clone()) != old.telefono).unwrap_or(false) {
            changed.push("telefono".to_string());
        }
        if request.correo.as_ref().map(|c| Some(c.clone()) != old.correo).unwrap_or(false) {
            changed.push("correo".to_string());
        }
        if request.fecha_nacimiento.as_ref().map(|f| Some(*f) != old.fecha_nacimiento).unwrap_or(false) {
            changed.push("fecha_nacimiento".to_string());
        }
        
        changed
    }
}
