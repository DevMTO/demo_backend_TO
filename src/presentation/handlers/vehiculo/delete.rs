//! DELETE handlers para Vehiculo

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_deleted;
use crate::presentation::extractors::AuthUser;

/// Eliminar (soft delete) un vehículo
#[instrument(skip(state, auth))]
pub async fn delete_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let _vehiculo = state.container.vehiculo_service
        .delete_vehiculo(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}

/// Eliminación permanente de un vehículo (hard delete)
#[instrument(skip(state, auth))]
pub async fn hard_delete_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let _vehiculo = state.container.vehiculo_service
        .hard_delete_vehiculo(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}
