//! DELETE handlers para Persona

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::{debug, info, instrument};

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_deleted;

/// Eliminar persona (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("[DELETE] Eliminando persona ID: {}", id);
    
    state.container.persona_service
        .delete_persona(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona eliminada ID: {}", id);
    Ok(json_deleted())
}

/// Eliminación permanente de persona (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente personas".to_string()));
    }
    
    debug!("[DELETE] HARD DELETE persona ID: {}", id);
    
    state.container.persona_service
        .hard_delete_persona(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona ELIMINADA PERMANENTEMENTE ID: {}", id);
    Ok(json_deleted())
}
