//! DELETE handlers para Pago

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_deleted;

/// Eliminar pago
#[instrument(skip(state, auth))]
pub async fn delete_pago(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.pago_service
        .delete_pago(
            id, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_deleted())
}
