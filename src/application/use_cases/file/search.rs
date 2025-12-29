use std::sync::Arc;
use chrono::NaiveDate;
use tracing::instrument;

use crate::application::dtos::FileResponse;
use crate::application::ports::FileRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct SearchFilesUseCase {
    file_repository: Arc<dyn FileRepositoryPort>,
}

impl SearchFilesUseCase {
    pub fn new(file_repository: Arc<dyn FileRepositoryPort>) -> Self {
        Self { file_repository }
    }
    
    /// Buscar files por rango de fechas
    #[instrument(skip(self))]
    pub async fn by_date_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository.find_by_date_range(from, to).await?;
        Ok(files.into_iter().map(FileResponse::from).collect())
    }
    
    /// Buscar files próximos (upcoming)
    #[instrument(skip(self))]
    pub async fn upcoming(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository.find_upcoming().await?;
        Ok(files.into_iter().map(FileResponse::from).collect())
    }
    
    /// Buscar files con pago pendiente
    #[instrument(skip(self))]
    pub async fn pending_payment(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository.find_pending_payment().await?;
        Ok(files.into_iter().map(FileResponse::from).collect())
    }
}
