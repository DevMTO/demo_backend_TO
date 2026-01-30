//! PUT handlers para Guia

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateGuiaRequest, GuiaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

#[instrument(skip(state, auth, request))]
pub async fn update_guia(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(request): Json<UpdateGuiaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_g = state.container.guia_service.get_guia(id).await?;
    let updated = request.apply_to(old_g, Some(auth.user.id));
    let result = state.container.guia_service.update_guia(id, &updated, auth.user.id, &auth.user.username).await?;
    Ok(json_ok(GuiaResponse::from(result)))
}
