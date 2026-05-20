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

use super::post::{can_manage_role, validate_entity_for_role, can_manage_target_user};

/// Actualizar un usuario existente
/// SuperAdmin, Admin, hoteles_gerente y agencias_gerente pueden editar usuarios
#[instrument(skip(state, auth, request))]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Check if user has permission to update users
    let can_update = matches!(
        auth.user.role,
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerenteCadena | UserRole::HotelesGerente | UserRole::AgenciasGerente
    );
    
    if !can_update {
        return Err(ApplicationError::Forbidden("No tienes permisos para editar usuarios".to_string()));
    }
    
    // Validate role permissions if role is being changed
    if let Some(ref target_role) = request.role {
        if !can_manage_role(&auth.user.role, target_role) {
            return Err(ApplicationError::Forbidden(format!("No puedes editar usuarios con rol '{}'", target_role)));
        }
        
        // Validate entity ownership for the new role
        if let Some(target_id_entidad) = request.id_entidad {
            validate_entity_for_role(
                &state,
                &auth.user.role,
                auth.user.id_entidad,
                target_role,
                Some(target_id_entidad),
            ).await?;
        }
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let result = state.container.user_service
        .update_user(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(result.user))
}

/// Cambiar contraseña de un usuario
/// SuperAdmin, Admin, HotelesGerenteCadena, HotelesGerente y AgenciasGerente pueden cambiar contraseñas en su ámbito
#[instrument(skip(state, auth, request))]
pub async fn admin_change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<AdminChangePasswordRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Check if user has permission to change passwords
    let can_change_password = matches!(
        auth.user.role,
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerenteCadena | UserRole::HotelesGerente | UserRole::AgenciasGerente
    );
    
    if !can_change_password {
        return Err(ApplicationError::Forbidden("No tienes permisos para cambiar contraseñas".to_string()));
    }
    
    // Get the target user to validate scope
    let target_user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // Validate that the manager has scope over this user
    can_manage_target_user(&state, &auth.user.role, auth.user.id_entidad, &target_user).await?;
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let user_dto = state.container.user_service
        .admin_change_password(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}
