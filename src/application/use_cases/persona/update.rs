use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{UpdatePersonaRequest, PersonaResponse};
use crate::application::ports::PersonaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct UpdatePersonaUseCase {
    persona_repository: Arc<dyn PersonaRepositoryPort>,
}

impl UpdatePersonaUseCase {
    pub fn new(persona_repository: Arc<dyn PersonaRepositoryPort>) -> Self {
        Self { persona_repository }
    }
    
    /// Ejecutar el caso de uso de actualización de persona
    /// 
    /// # Validaciones de negocio:
    /// - Verifica que la persona exista
    /// - Si se cambia el documento, verifica que no esté duplicado
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        id: i32,
        request: UpdatePersonaRequest,
        user_id: i32,
    ) -> Result<PersonaResponse, ApplicationError> {
        // Verificar que existe
        let existing = self.persona_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona {} no encontrada", id)))?;
        
        // Si se están cambiando documento, verificar unicidad
        if let (Some(tipo), Some(nro)) = (&request.tipo_documento, &request.nro_documento) {
            // Solo verificar si cambió
            if tipo != &existing.tipo_documento.to_string() || nro != &existing.nro_documento {
                if self.persona_repository.exists_by_documento(tipo, nro).await? {
                    return Err(ApplicationError::Conflict(
                        format!("Ya existe una persona con {} {}", tipo, nro)
                    ));
                }
            }
        }
        
        // Aplicar cambios
        let updated_entity = request.apply_to(existing, Some(user_id));
        
        // Persistir
        let result = self.persona_repository.update(&updated_entity).await?;
        
        info!("✏️ Persona actualizada: {} {} (ID: {})", result.nombre, result.apellidos, result.id);
        
        Ok(PersonaResponse::from(result))
    }
}
