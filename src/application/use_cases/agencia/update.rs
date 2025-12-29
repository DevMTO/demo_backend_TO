use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{UpdateAgenciaRequest, AgenciaResponse};
use crate::application::ports::AgenciaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct UpdateAgenciaUseCase {
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
}

impl UpdateAgenciaUseCase {
    pub fn new(agencia_repository: Arc<dyn AgenciaRepositoryPort>) -> Self {
        Self { agencia_repository }
    }
    
    /// Ejecutar el caso de uso de actualización de agencia
    /// 
    /// # Validaciones de negocio:
    /// - Verifica que la agencia exista
    /// - Si se cambia el RUC, verifica que no esté duplicado
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        id: i32,
        request: UpdateAgenciaRequest,
        user_id: i32,
    ) -> Result<AgenciaResponse, ApplicationError> {
        // Verificar que existe
        let existing = self.agencia_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
        
        // Si se está cambiando el RUC, verificar unicidad
        if let Some(ref new_ruc) = request.ruc {
            if new_ruc != &existing.ruc {
                if self.agencia_repository.exists_by_ruc(new_ruc).await? {
                    return Err(ApplicationError::Conflict(
                        format!("Ya existe una agencia con RUC {}", new_ruc)
                    ));
                }
            }
        }
        
        // Aplicar cambios
        let updated_entity = request.apply_to(existing, Some(user_id));
        
        // Persistir
        let result = self.agencia_repository.update(&updated_entity).await?;
        
        info!("✏️ Agencia actualizada: {} (ID: {})", result.nombre, result.id);
        
        Ok(AgenciaResponse::from(result))
    }
}
