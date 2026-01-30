//! PUT handlers para Restaurante

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::UpdateRestauranteRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/v1/restaurantes/:id - Actualizar un restaurante
#[instrument(skip(state, auth, request))]
pub async fn update_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRestauranteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let updated = state.container.restaurante_service
        .update_restaurante(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Restaurante actualizado: {} (ID: {})", updated.nombre, id);
    Ok(json_ok(updated))
}
