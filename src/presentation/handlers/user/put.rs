//! PUT handlers para User

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{UpdateUserRequest, AdminChangePasswordRequest};

/// Actualizar un usuario existente
/// Solo SuperAdmin puede editar usuarios
#[instrument(skip(state, auth, request))]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede editar usuarios".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let result = state.container.user_service
        .update_user(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(result.user))
}

/// Cambiar contraseña de un usuario (solo SuperAdmin)
#[instrument(skip(state, auth, request))]
pub async fn admin_change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<AdminChangePasswordRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede cambiar contraseñas de otros usuarios".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let user_dto = state.container.user_service
        .admin_change_password(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}
