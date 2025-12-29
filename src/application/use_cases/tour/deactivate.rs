use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::ports::TourRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct DeactivateTourUseCase {
    tour_repository: Arc<dyn TourRepositoryPort>,
}

impl DeactivateTourUseCase {
    pub fn new(tour_repository: Arc<dyn TourRepositoryPort>) -> Self {
        Self { tour_repository }
    }
    
    /// Ejecutar soft delete de tour
    #[instrument(skip(self))]
    pub async fn execute(&self, id: i32, user_id: i32) -> Result<(), ApplicationError> {
        let deleted = self.tour_repository.soft_delete(id, user_id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("Tour {} no encontrado", id)));
        }
        
        info!("🗑️ Tour {} desactivado por usuario {}", id, user_id);
        Ok(())
    }
}

pub struct RestoreTourUseCase {
    tour_repository: Arc<dyn TourRepositoryPort>,
}

impl RestoreTourUseCase {
    pub fn new(tour_repository: Arc<dyn TourRepositoryPort>) -> Self {
        Self { tour_repository }
    }
    
    /// Ejecutar restauración de tour
    #[instrument(skip(self))]
    pub async fn execute(&self, id: i32, user_id: i32) -> Result<(), ApplicationError> {
        let restored = self.tour_repository.restore(id, user_id).await?;
        
        if !restored {
            return Err(ApplicationError::NotFound(format!("Tour {} no encontrado", id)));
        }
        
        info!("♻️ Tour {} restaurado por usuario {}", id, user_id);
        Ok(())
    }
}
