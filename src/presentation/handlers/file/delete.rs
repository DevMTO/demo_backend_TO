//! DELETE handlers para File

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_deleted;

/// Eliminar file (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar permisos
    let file = state.container.file_service.get_file(id).await?;
    if !auth.user.role.is_admin() {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        let check_cadena = if auth.user.role == UserRole::HotelesGerente {
            if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(file.id_entidad).await {
                hotel.id_cadena == user_entidad
            } else {
                false
            }
        } else {
            false
        };

        if file.id_entidad != user_entidad && !check_cadena {
            return Err(ApplicationError::Forbidden("No tienes permiso para eliminar este file".to_string()));
        }
    }

    state.container.file_service
        .delete_file(
            id, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_deleted())
}

/// Eliminación permanente de file (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente files".to_string()));
    }
    
    state.container.file_service
        .hard_delete_file(
            id, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_deleted())
}
