use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreateTourRequest, TourResponse};
use crate::application::ports::TourRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct CreateTourUseCase {
    tour_repository: Arc<dyn TourRepositoryPort>,
}

impl CreateTourUseCase {
    pub fn new(tour_repository: Arc<dyn TourRepositoryPort>) -> Self {
        Self { tour_repository }
    }
    
    /// Ejecutar el caso de uso de creación de tour
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        request: CreateTourRequest,
        user_id: i32,
    ) -> Result<TourResponse, ApplicationError> {
        // Crear entidad de dominio
        let tour = request.into_entity(Some(user_id));
        
        // Persistir
        let created = self.tour_repository.create(&tour).await?;
        
        info!("✅ Tour creado: {} (ID: {})", created.nombre, created.id);
        
        Ok(TourResponse::from(created))
    }
}
