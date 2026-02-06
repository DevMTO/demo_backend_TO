//! PATCH handlers para Entrada

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

#[instrument(skip(state, auth))]
pub async fn restore_entrada(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_service.restore_entrada(id, auth.user.id, &auth.user.username).await?;
    
    Ok(json_message("Entrada restaurada"))
}
