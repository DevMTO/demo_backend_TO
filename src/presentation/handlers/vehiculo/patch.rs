//! PATCH handlers para Vehiculo

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;

/// Restaurar un vehículo eliminado (soft delete)
#[instrument(skip(state, auth))]
pub async fn restore_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.vehiculo_service
        .restore_vehiculo(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(()))
}
