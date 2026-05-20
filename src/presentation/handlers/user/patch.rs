//! PATCH handlers para User

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;

use super::post::can_manage_target_user;

/// Activar un usuario
/// SuperAdmin, Admin, HotelesGerenteCadena, HotelesGerente y AgenciasGerente pueden activar usuarios en su ámbito
#[instrument(skip(state, auth))]
pub async fn activate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Check if user has permission to activate users
    let can_activate = matches!(
        auth.user.role,
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerenteCadena | UserRole::HotelesGerente | UserRole::AgenciasGerente
    );
    
    if !can_activate {
        return Err(ApplicationError::Forbidden("No tienes permisos para activar usuarios".to_string()));
    }
    
    // Get the target user to validate scope
    let target_user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // Validate that the manager has scope over this user
    can_manage_target_user(&state, &auth.user.role, auth.user.id_entidad, &target_user).await?;
    
    let (user_dto, _old_active) = state.container.user_service
        .activate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}

/// Desactivar un usuario
/// SuperAdmin, Admin, HotelesGerenteCadena, HotelesGerente y AgenciasGerente pueden desactivar usuarios en su ámbito
#[instrument(skip(state, auth))]
pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Check if user has permission to deactivate users
    let can_deactivate = matches!(
        auth.user.role,
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerenteCadena | UserRole::HotelesGerente | UserRole::AgenciasGerente
    );
    
    if !can_deactivate {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar usuarios".to_string()));
    }
    
    // Get the target user to validate scope
    let target_user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // Validate that the manager has scope over this user
    can_manage_target_user(&state, &auth.user.role, auth.user.id_entidad, &target_user).await?;
    
    let (user_dto, _old_active) = state.container.user_service
        .deactivate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}