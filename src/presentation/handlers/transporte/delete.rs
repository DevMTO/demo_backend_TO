//! DELETE handlers para Transporte

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// DELETE /api/v1/transportes/:id - Desactivar un transporte (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.transporte_service
        .deactivate_transporte(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Transporte desactivado (ID: {})", id);
    Ok(json_message("Transporte desactivado"))
}

/// DELETE /api/v1/transportes/:id/hard-delete - Eliminación permanente (SOLO SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn hard_delete_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin) {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente transportes".to_string()));
    }
    
    state.container.transporte_service
        .hard_delete_transporte(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("🗑️ Handler: Transporte ELIMINADO PERMANENTEMENTE (ID: {})", id);
    Ok(json_message("Transporte eliminado permanentemente"))
}
