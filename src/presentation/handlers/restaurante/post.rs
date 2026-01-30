//! POST handlers para Restaurante

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::CreateRestauranteRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/restaurantes - Crear un nuevo restaurante
#[instrument(skip(state, auth, request))]
pub async fn create_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateRestauranteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let created = state.container.restaurante_service
        .create_restaurante(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Restaurante creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(created))
}
