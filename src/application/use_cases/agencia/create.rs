use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreateAgenciaRequest, AgenciaResponse};
use crate::application::ports::AgenciaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct CreateAgenciaUseCase {
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
}

impl CreateAgenciaUseCase {
    pub fn new(agencia_repository: Arc<dyn AgenciaRepositoryPort>) -> Self {
        Self { agencia_repository }
    }
    
    /// Ejecutar el caso de uso de creación de agencia
    /// 
    /// # Validaciones de negocio:
    /// - Verifica que no exista una agencia con el mismo RUC
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        request: CreateAgenciaRequest,
        user_id: i32,
    ) -> Result<AgenciaResponse, ApplicationError> {
        // Validación de negocio: RUC único
        if self.agencia_repository.exists_by_ruc(&request.ruc).await? {
            return Err(ApplicationError::Conflict(
                format!("Ya existe una agencia con RUC {}", request.ruc)
            ));
        }
        
        // Crear entidad de dominio
        let agencia = request.into_entity(Some(user_id));
        
        // Persistir
        let created = self.agencia_repository.create(&agencia).await?;
        
        info!("✅ Agencia creada: {} (ID: {})", created.nombre, created.id);
        
        Ok(AgenciaResponse::from(created))
    }
}
