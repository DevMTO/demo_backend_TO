//! PATCH handlers para Restaurante

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// POST /api/v1/restaurantes/:id/restore - Restaurar un restaurante desactivado
#[instrument(skip(state, auth))]
pub async fn restore_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.restaurante_service
        .restore_restaurante(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("♻️ Handler: Restaurante restaurado (ID: {})", id);
    Ok(json_message("Restaurante restaurado correctamente"))
}
