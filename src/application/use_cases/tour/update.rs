use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{UpdateTourRequest, TourResponse};
use crate::application::ports::TourRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct UpdateTourUseCase {
    tour_repository: Arc<dyn TourRepositoryPort>,
}

impl UpdateTourUseCase {
    pub fn new(tour_repository: Arc<dyn TourRepositoryPort>) -> Self {
        Self { tour_repository }
    }
    
    /// Ejecutar el caso de uso de actualización de tour
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        id: i32,
        request: UpdateTourRequest,
        user_id: i32,
    ) -> Result<TourResponse, ApplicationError> {
        // Verificar que existe
        let existing = self.tour_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
        
        // Aplicar cambios
        let updated_entity = request.apply_to(existing, Some(user_id));
        
        // Persistir
        let result = self.tour_repository.update(&updated_entity).await?;
        
        info!("✏️ Tour actualizado: {} (ID: {})", result.nombre, result.id);
        
        Ok(TourResponse::from(result))
    }
}
