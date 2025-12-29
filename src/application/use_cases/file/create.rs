use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreateFileRequest, FileResponse};
use crate::application::ports::FileRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct CreateFileUseCase {
    file_repository: Arc<dyn FileRepositoryPort>,
}

impl CreateFileUseCase {
    pub fn new(file_repository: Arc<dyn FileRepositoryPort>) -> Self {
        Self { file_repository }
    }
    
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        request: CreateFileRequest,
        user_id: i32,
    ) -> Result<FileResponse, ApplicationError> {
        let file = request.into_entity(Some(user_id));
        
        let created = self.file_repository.create(&file).await?;
        
        info!("✅ File creado (ID: {})", created.id);
        
        Ok(FileResponse::from(created))
    }
}
