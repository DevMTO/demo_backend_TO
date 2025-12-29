use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::ports::AgenciaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct DeactivateAgenciaUseCase {
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
}

impl DeactivateAgenciaUseCase {
    pub fn new(agencia_repository: Arc<dyn AgenciaRepositoryPort>) -> Self {
        Self { agencia_repository }
    }
    
    /// Ejecutar soft delete de agencia
    #[instrument(skip(self))]
    pub async fn execute(&self, id: i32, user_id: i32) -> Result<(), ApplicationError> {
        let deleted = self.agencia_repository.soft_delete(id, user_id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Agencia {} no encontrada", id)));
        }
        
        info!("🗑️ Agencia {} desactivada por usuario {}", id, user_id);
        Ok(())
    }
}

pub struct RestoreAgenciaUseCase {
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
}

impl RestoreAgenciaUseCase {
    pub fn new(agencia_repository: Arc<dyn AgenciaRepositoryPort>) -> Self {
        Self { agencia_repository }
    }
    
    /// Ejecutar restauración de agencia
    #[instrument(skip(self))]
    pub async fn execute(&self, id: i32, user_id: i32) -> Result<(), ApplicationError> {
        let restored = self.agencia_repository.restore(id, user_id).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Agencia {} no encontrada", id)));
        }
        
        info!("♻️ Agencia {} restaurada por usuario {}", id, user_id);
        Ok(())
    }
}
