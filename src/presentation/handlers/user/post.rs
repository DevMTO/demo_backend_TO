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

pub fn can_manage_role(manager_role: &UserRole, target_role: &str) -> bool {
    match manager_role {
        UserRole::SuperAdmin | UserRole::Admin => true,
        UserRole::HotelesGerente => target_role == "hoteles_gerente" || target_role == "hoteles",
        UserRole::AgenciasGerente => target_role == "agencias_gerente" || target_role == "agencias" || target_role == "agencias_contador",
        _ => false,
    }
}

pub fn validate_entity_for_role(
    manager_role: &UserRole,
    manager_id_entidad: Option<i32>,
    target_role: &str,
    target_id_entidad: Option<i32>,
) -> Result<(), ApplicationError> {
    // No entity required for roles that don't need one
    let role_needs_entidad = matches!(
        target_role,
        "agencias" | "agencias_contador" | "agencias_gerente" | 
        "hoteles" | "hoteles_gerente" | 
        "transportes" | "restaurantes"
    );
    
    if !role_needs_entidad {
        return Ok(());
    }
    
    // If target has no entidad, allow (will be validated separately if needed)
    let target_entidad = match target_id_entidad {
        Some(e) => e,
        None => return Ok(()),
    };
    
    match manager_role {
        UserRole::SuperAdmin | UserRole::Admin => Ok(()),
        UserRole::HotelesGerente => {
            // Target must be a hotel belonging to the manager's cadena
            if target_role == "hoteles" {
                if manager_id_entidad.is_some() {
                    // We'll validate the hotel belongs to the cadena in the service layer
                    // by checking the hotel's id_cadena
                    return Ok(());
                }
            }
            // Hoteles_gerente can only manage hotels
            if target_role == "hoteles_gerente" {
                // Creating another gerente - entity must match
                if target_entidad != manager_id_entidad.unwrap_or(0) {
                    return Err(ApplicationError::Forbidden("No puedes crear un gerente de cadena para una cadena diferente".to_string()));
                }
            }
            Ok(())
        },
        UserRole::AgenciasGerente => {
            // Target must belong to the manager's agencia
            if let Some(manager_entidad) = manager_id_entidad {
                if target_entidad != manager_entidad {
                    return Err(ApplicationError::Forbidden("No puedes crear usuarios para una agencia diferente".to_string()));
                }
            }
            Ok(())
        },
        _ => Err(ApplicationError::Forbidden("No tienes permisos para crear usuarios de este rol".to_string())),
    }
}

/// Crear un nuevo usuario (opcionalmente con persona nueva)
/// Solo SuperAdmin, Admin, hoteles_gerente y agencias_gerente pueden crear usuarios
#[instrument(skip(state, auth, request))]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Check if user has permission to create users
    let can_create = matches!(
        auth.user.role,
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente | UserRole::AgenciasGerente
    );
    
    if !can_create {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear usuarios".to_string()));
    }
    
    // Validate role permissions
    if !can_manage_role(&auth.user.role, &request.role) {
        return Err(ApplicationError::Forbidden(format!("No puedes crear usuarios con rol '{}'", request.role)));
    }
    
    // Validate entity ownership
    validate_entity_for_role(
        &auth.user.role,
        auth.user.id_entidad,
        &request.role,
        request.id_entidad,
    )?;
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let result = state.container.user_service
        .create_user(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(result.user))
}
