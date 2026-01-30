//! POST handlers para Entrada

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateEntradaRequest, EntradaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

#[instrument(skip(state, auth, request))]
pub async fn create_entrada(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreateEntradaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let entity = request.into_entity(Some(auth.user.id));
    let created = state.container.entrada_service.create_entrada(&entity, auth.user.id, &auth.user.username).await?;
    
    // Inicializar precios por defecto para la nueva entrada
    let _ = state.container.entrada_precio_service
        .initialize_default_precios(created.id, Some(auth.user.id))
        .await;
    
    Ok(json_created(EntradaResponse::from(created)))
}
