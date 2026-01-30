//! DELETE handlers para User

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_deleted;
use crate::presentation::extractors::AuthUser;

/// Eliminar (desactivar) un usuario
/// Solo SuperAdmin puede eliminar usuarios
#[instrument(skip(state, auth))]
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar usuarios".to_string()));
    }
    
    let _user = state.container.user_service
        .delete_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}

/// Eliminación permanente de un usuario (hard delete)
/// Solo SuperAdmin puede ejecutar esta acción
#[instrument(skip(state, auth))]
pub async fn hard_delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente usuarios".to_string()));
    }
    
    let _user = state.container.user_service
        .hard_delete_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}
