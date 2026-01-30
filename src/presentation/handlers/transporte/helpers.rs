//! Helper functions para Transporte handlers

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

/// Busca el ID del transporte asociado al usuario
pub async fn find_user_transporte_id(state: &AppState, auth: &AuthUser) -> Result<i32, ApplicationError> {
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            return Ok(id_entidad);
        }
    }
    
    if let Some(persona_id) = auth.user.id_persona {
        if let Some(transporte) = state.container.transporte_service.find_by_encargado(persona_id).await? {
            return Ok(transporte.id);
        }
    }
    
    Err(ApplicationError::NotFound("No tienes un transporte asociado".to_string()))
}
