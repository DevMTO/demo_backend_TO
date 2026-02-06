//! PUT handlers para Entrada

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateEntradaRequest, EntradaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

#[instrument(skip(state, auth, request))]
pub async fn update_entrada(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(request): Json<UpdateEntradaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_e = state.container.entrada_service.get_entrada(id).await?;
    let updated = request.apply_to(old_e, Some(auth.user.id));
    let result = state.container.entrada_service.update_entrada(id, &updated, auth.user.id, &auth.user.username).await?;
    
    Ok(json_ok(EntradaResponse::from(result)))
}
