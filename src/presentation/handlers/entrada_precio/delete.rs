//! DELETE handlers para EntradaPrecio

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_message;

/// DELETE /api/entrada-precios/:id
/// Eliminar un precio de entrada
#[instrument(skip(state, _auth))]
pub async fn delete_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_precio_service.delete_precio(id).await?;
    Ok(json_message("Precio eliminado"))
}
