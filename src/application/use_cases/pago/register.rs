use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreatePagoRequest, PagoResponse};
use crate::application::ports::{PagoRepositoryPort, FileRepositoryPort};
use crate::domain::errors::ApplicationError;

pub struct RegisterPagoUseCase {
    pago_repository: Arc<dyn PagoRepositoryPort>,
    file_repository: Arc<dyn FileRepositoryPort>,
}

impl RegisterPagoUseCase {
    pub fn new(
        pago_repository: Arc<dyn PagoRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
    ) -> Self {
        Self { pago_repository, file_repository }
    }
    
    /// Ejecutar el caso de uso de registro de pago
    /// 
    /// # Validaciones de negocio:
    /// - Verifica que el file exista
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        request: CreatePagoRequest,
        user_id: i32,
    ) -> Result<PagoResponse, ApplicationError> {
        // Validación de negocio: el file debe existir
        let _ = self.file_repository
            .find_by_id(request.id_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        
        // Crear entidad de dominio
        let pago = request.into_entity(Some(user_id));
        
        // Persistir
        let created = self.pago_repository.create(&pago).await?;
        
        info!("✅ Pago registrado: {} {} (ID: {})", created.tipo_movimiento, created.monto, created.id);
        
        Ok(PagoResponse::from(created))
    }
}
