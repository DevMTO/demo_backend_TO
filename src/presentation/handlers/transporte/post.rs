//! POST handlers para Transporte

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::CreateTransporteRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/transportes - Crear un nuevo transporte
#[instrument(skip(state, auth, request))]
pub async fn create_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let created = state.container.transporte_service
        .create_transporte(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Transporte creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(created))
}
