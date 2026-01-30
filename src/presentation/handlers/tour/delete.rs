//! DELETE handlers para Tour

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_message, json_deleted};

/// Desactivar tour (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.tour_service
        .deactivate_tour(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("[DELETE] Tour desactivado: ID {}", id);
    Ok(json_message("Tour desactivado correctamente"))
}

/// Eliminación permanente de tour (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente tours".to_string()));
    }
    
    state.container.tour_service
        .hard_delete_tour(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("[DELETE] Tour ELIMINADO PERMANENTEMENTE: ID {}", id);
    Ok(json_deleted())
}
