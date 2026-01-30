//! DELETE handlers para Conductor

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_deleted;

/// DELETE /api/v1/conductores/:id - Eliminar un conductor (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.conductor_service
        .delete_conductor(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Conductor eliminado (ID: {})", id);
    Ok(json_deleted())
}

/// DELETE /api/v1/conductores/:id/hard-delete - Eliminación permanente (Solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn hard_delete_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente conductores".to_string()));
    }
    
    // Delegar al servicio
    state.container.conductor_service
        .hard_delete_conductor(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Conductor ELIMINADO PERMANENTEMENTE (ID: {})", id);
    Ok(json_deleted())
}
