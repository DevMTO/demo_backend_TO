use std::sync::Arc;
use tracing::instrument;

use crate::application::dtos::PersonaResponse;
use crate::application::ports::PersonaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct SearchPersonasUseCase {
    persona_repository: Arc<dyn PersonaRepositoryPort>,
}

impl SearchPersonasUseCase {
    pub fn new(persona_repository: Arc<dyn PersonaRepositoryPort>) -> Self {
        Self { persona_repository }
    }
    
    /// Buscar personas por texto (nombre, apellidos, documento)
    #[instrument(skip(self))]
    pub async fn execute(&self, query: &str) -> Result<Vec<PersonaResponse>, ApplicationError> {
        let personas = self.persona_repository.search(query).await?;
        Ok(personas.into_iter().map(PersonaResponse::from).collect())
    }
}
