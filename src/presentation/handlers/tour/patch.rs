//! PATCH handlers para Tour

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// Restaurar tour desactivado
#[instrument(skip(state, auth))]
pub async fn restore_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.tour_service
        .restore_tour(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Tour restaurado: ID {}", id);
    Ok(json_message("Tour restaurado correctamente"))
}
