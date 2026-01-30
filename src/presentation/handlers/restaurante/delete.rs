//! DELETE handlers para Restaurante

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_message, json_deleted};

/// DELETE /api/v1/restaurantes/:id - Desactivar un restaurante (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.restaurante_service
        .deactivate_restaurante(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Restaurante desactivado (ID: {})", id);
    Ok(json_message("Restaurante desactivado correctamente"))
}

/// DELETE /api/v1/restaurantes/:id/hard-delete - Eliminación permanente (Solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn hard_delete_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente restaurantes".to_string()));
    }
    
    state.container.restaurante_service
        .hard_delete_restaurante(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Restaurante ELIMINADO PERMANENTEMENTE (ID: {})", id);
    Ok(json_deleted())
}
