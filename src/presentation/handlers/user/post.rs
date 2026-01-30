//! POST handlers para User

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_created;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::CreateUserRequest;

/// Crear un nuevo usuario (opcionalmente con persona nueva)
/// Solo SuperAdmin puede crear usuarios
#[instrument(skip(state, auth, request))]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede crear usuarios".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let result = state.container.user_service
        .create_user(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(result.user))
}
