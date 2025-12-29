use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{UpdateFileRequest, FileResponse};
use crate::application::ports::FileRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct UpdateFileUseCase {
    file_repository: Arc<dyn FileRepositoryPort>,
}

impl UpdateFileUseCase {
    pub fn new(file_repository: Arc<dyn FileRepositoryPort>) -> Self {
        Self { file_repository }
    }
    
    /// Ejecutar el caso de uso de actualización de file
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        id: i32,
        request: UpdateFileRequest,
        user_id: i32,
    ) -> Result<FileResponse, ApplicationError> {
        // Verificar que existe
        let existing = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Aplicar cambios
        let updated_entity = request.apply_to(existing, Some(user_id));
        
        // Persistir
        let result = self.file_repository.update(&updated_entity).await?;
        
        info!("✏️ File actualizado (ID: {})", result.id);
        
        Ok(FileResponse::from(result))
    }
}
