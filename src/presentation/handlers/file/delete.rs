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
