//! DELETE handlers para Guia

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_deleted;

#[instrument(skip(state, auth))]
pub async fn delete_guia(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.guia_service.delete_guia(id, auth.user.id, &auth.user.username).await?;
    Ok(json_deleted())
}

/// Eliminación permanente de guía (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_guia(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente guías".to_string()));
    }
    state.container.guia_service.hard_delete_guia(id, auth.user.id, &auth.user.username).await?;
    Ok(json_deleted())
}
