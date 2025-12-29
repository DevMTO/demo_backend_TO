use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreatePersonaRequest, PersonaResponse};
use crate::application::ports::PersonaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct CreatePersonaUseCase {
    persona_repository: Arc<dyn PersonaRepositoryPort>,
}

impl CreatePersonaUseCase {
    pub fn new(persona_repository: Arc<dyn PersonaRepositoryPort>) -> Self {
        Self { persona_repository }
    }
    
    /// Ejecutar el caso de uso de creación de persona
    /// 
    /// # Validaciones de negocio:
    /// - Verifica que no exista una persona con el mismo documento
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        request: CreatePersonaRequest,
        user_id: i32,
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
        let persona = request.into_entity(Some(user_id));
        
        // Persistir
        let created = self.persona_repository.create(&persona).await?;
        
        info!("✅ Persona creada: {} {} (ID: {})", created.nombre, created.apellidos, created.id);
        
        Ok(PersonaResponse::from(created))
    }
}
