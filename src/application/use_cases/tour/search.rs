use std::sync::Arc;
use tracing::instrument;

use crate::application::dtos::TourResponse;
use crate::application::ports::TourRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct SearchToursUseCase {
    tour_repository: Arc<dyn TourRepositoryPort>,
}

impl SearchToursUseCase {
    pub fn new(tour_repository: Arc<dyn TourRepositoryPort>) -> Self {
        Self { tour_repository }
    }
    
    /// Buscar tours por nombre
    #[instrument(skip(self))]
    pub async fn by_name(&self, nombre: &str) -> Result<Vec<TourResponse>, ApplicationError> {
        let tours = self.tour_repository.find_by_nombre(nombre).await?;
        Ok(tours.into_iter().map(TourResponse::from).collect())
    }
    
    /// Buscar tours por duración
    #[instrument(skip(self))]
    pub async fn by_duracion(&self, days: i32) -> Result<Vec<TourResponse>, ApplicationError> {
        let tours = self.tour_repository.find_by_duracion(days).await?;
        Ok(tours.into_iter().map(TourResponse::from).collect())
    }
}
