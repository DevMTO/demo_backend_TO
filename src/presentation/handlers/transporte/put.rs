//! PUT handlers para Transporte

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::UpdateTransporteRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

use super::helpers::find_user_transporte_id;

/// PUT /api/v1/transportes/:id - Actualizar un transporte (por admin)
#[instrument(skip(state, auth, request))]
pub async fn update_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let updated = state.container.transporte_service
        .update_transporte(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Transporte actualizado: {} (ID: {})", updated.nombre, id);
    Ok(json_ok(updated))
}

/// PUT /api/v1/transportes/me - Actualizar mi propio transporte
#[instrument(skip(state, auth, request))]
pub async fn update_mi_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚐 Usuario '{}' intenta actualizar su transporte", auth.user.username);
    
    let transporte_id = find_user_transporte_id(&state, &auth).await?;
    
    let result = state.container.transporte_service
        .update_my_transporte(transporte_id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Transporte actualizado por su usuario: {}", result.nombre);
    Ok(json_ok(result))
}
