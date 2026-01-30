//! POST handlers para Guia

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateGuiaRequest, GuiaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

#[instrument(skip(state, auth, request))]
pub async fn create_guia(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreateGuiaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let entity = request.into_entity(Some(auth.user.id));
    let created = state.container.guia_service.create_guia(&entity, auth.user.id, &auth.user.username).await?;
    Ok(json_created(GuiaResponse::from(created)))
}
