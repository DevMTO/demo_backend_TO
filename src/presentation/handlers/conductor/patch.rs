//! PATCH handlers para Conductor

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// PATCH /api/v1/conductores/:id/restore - Restaurar un conductor desactivado
#[instrument(skip(state, auth))]
pub async fn restore_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.conductor_service
        .restore_conductor(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("♻️ Handler: Conductor restaurado (ID: {})", id);
    Ok(json_message("Conductor restaurado"))
}
