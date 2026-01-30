//! POST handlers para Agencias

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreateAgenciaRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/v1/agencias - Crear nueva agencia
#[instrument(skip(state, auth, request))]
pub async fn create_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.agencia_service
        .create_agencia(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(response))
}
