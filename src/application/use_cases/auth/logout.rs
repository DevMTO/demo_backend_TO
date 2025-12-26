//! # Logout Use Case
//! 
//! Caso de uso para cerrar sesión.

use std::sync::Arc;

use crate::domain::errors::ApplicationError;
use crate::application::ports::SessionRepositoryPort;
use crate::application::dtos::auth_dto::LogoutRequest;

/// Use case para logout
pub struct LogoutUseCase {
    session_repository: Arc<dyn SessionRepositoryPort>,
}

impl LogoutUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepositoryPort>) -> Self {
        Self { session_repository }
    }
    
    /// Ejecutar el caso de uso de logout
    pub async fn execute(
        &self,
        user_id: i32,
        session_id: i32,
        request: LogoutRequest,
    ) -> Result<u64, ApplicationError> {
        if request.all_sessions {
            // Revocar todas las sesiones del usuario
            let count = self.session_repository
                .delete_by_user_id(user_id)
                .await?;
            Ok(count)
        } else {
            // Solo revocar la sesión actual
            self.session_repository
                .revoke(session_id, "Logout manual")
                .await?;
            Ok(1)
        }
    }
}
