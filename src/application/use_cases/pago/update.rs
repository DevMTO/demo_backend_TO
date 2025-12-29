use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{UpdatePagoRequest, PagoResponse};
use crate::application::ports::PagoRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct UpdatePagoUseCase {
    pago_repository: Arc<dyn PagoRepositoryPort>,
}

impl UpdatePagoUseCase {
    pub fn new(pago_repository: Arc<dyn PagoRepositoryPort>) -> Self {
        Self { pago_repository }
    }
    
    /// Ejecutar el caso de uso de actualización de pago
    #[instrument(skip(self, request))]
    pub async fn execute(
        &self,
        id: i32,
        request: UpdatePagoRequest,
        user_id: i32,
    ) -> Result<PagoResponse, ApplicationError> {
        // Verificar que existe
        let existing = self.pago_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))?;
        
        // Aplicar cambios
        let updated_entity = request.apply_to(existing, Some(user_id));
        
        // Persistir
        let result = self.pago_repository.update(&updated_entity).await?;
        
        info!("✏️ Pago actualizado: {} {} (ID: {})", result.tipo_movimiento, result.monto, result.id);
        
        Ok(PagoResponse::from(result))
    }
}
