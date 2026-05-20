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
        UserRole::HotelesGerenteCadena => target_role == "hoteles_gerente" || target_role == "hoteles",
        UserRole::HotelesGerente => target_role == "hoteles",
        UserRole::AgenciasGerente => target_role == "agencias_gerente" || target_role == "agencias" || target_role == "agencias_contador",
        _ => false,
    }
}

/// Validate entity ownership for the target user being created.
/// This ensures managers can only create users within their own entity hierarchy.
pub async fn validate_entity_for_role(
    state: &AppState,
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
        UserRole::HotelesGerenteCadena => {
            // Target must be a hotel belonging to the manager's cadena
            if target_role == "hoteles" || target_role == "hoteles_gerente" {
                if let Some(cadena_id) = manager_id_entidad {
                    let hotel = state.container.hotel_repository.find_by_id(target_entidad).await?;
                    if let Some(h) = hotel {
                        if h.id_cadena != cadena_id {
                            return Err(ApplicationError::Forbidden(
                                "No puedes crear usuarios para hoteles que no pertenecen a tu cadena hotelera".to_string()
                            ));
                        }
                    } else {
                        return Err(ApplicationError::NotFound(
                            format!("Hotel con ID {} no encontrado", target_entidad)
                        ));
                    }
                }
            }
            Ok(())
        },
        UserRole::HotelesGerente => {
            // Hotel manager can only create users in their own hotel
            if target_role == "hoteles" {
                if target_entidad != manager_id_entidad.unwrap_or(0) {
                    return Err(ApplicationError::Forbidden("No puedes crear usuarios para un hotel diferente".to_string()));
                }
            }
            Ok(())
        },
        UserRole::AgenciasGerente => {
            // Target must belong to the manager's agencia
            if let Some(manager_entidad) = manager_id_entidad {
                // For agencias roles, validate the agencia belongs to the manager
                if target_role == "agencias" || target_role == "agencias_gerente" || target_role == "agencias_contador" {
                    let agencia = state.container.agencia_repository.find_by_id(target_entidad).await?;
                    if let Some(a) = agencia {
                        if a.id != manager_entidad {
                            return Err(ApplicationError::Forbidden(
                                "No puedes crear usuarios para agencias que no pertenecen a tu agencia".to_string()
                            ));
                        }
                    } else {
                        return Err(ApplicationError::NotFound(
                            format!("Agencia con ID {} no encontrada", target_entidad)
                        ));
                    }
                }
            }
            Ok(())
        },
        _ => Err(ApplicationError::Forbidden("No tienes permisos para crear usuarios de este rol".to_string())),
    }
}

/// Check if a manager can manage (activate/deactivate/change password) a target user.
/// Returns Ok(()) if allowed, or error if not.
pub async fn can_manage_target_user(
    state: &AppState,
    manager_role: &UserRole,
    manager_id_entidad: Option<i32>,
    target_user: &crate::domain::entities::User,
) -> Result<(), ApplicationError> {
    // SuperAdmin and Admin can manage any user
    match manager_role {
        UserRole::SuperAdmin | UserRole::Admin => Ok(()),
        
        UserRole::HotelesGerenteCadena => {
            // Manager can only manage users in their cadena
            if let Some(cadena_id) = manager_id_entidad {
                if let Some(target_id_entidad) = target_user.id_entidad {
                    // Get the hotel and check its cadena
                    if let Some(hotel) = state.container.hotel_repository.find_by_id(target_id_entidad).await? {
                        if hotel.id_cadena != cadena_id {
                            return Err(ApplicationError::Forbidden(
                                "Solo puedes gestionar usuarios de tu cadena hotelera".to_string()
                            ));
                        }
                    } else {
                        // Target user doesn't have a valid hotel entity, check if they're a cadena manager themselves
                        if target_user.role == UserRole::HotelesGerenteCadena && target_id_entidad != cadena_id {
                            return Err(ApplicationError::Forbidden(
                                "Solo puedes gestionar usuarios de tu cadena hotelera".to_string()
                            ));
                        }
                    }
                }
            }
            Ok(())
        },
        
        UserRole::HotelesGerente => {
            // Manager can only manage users in their hotel
            if let Some(manager_hotel_id) = manager_id_entidad {
                if let Some(target_id_entidad) = target_user.id_entidad {
                    if target_id_entidad != manager_hotel_id {
                        return Err(ApplicationError::Forbidden(
                            "Solo puedes gestionar usuarios de tu hotel".to_string()
                        ));
                    }
                }
            }
            Ok(())
        },
        
        UserRole::AgenciasGerente => {
            // Manager can only manage users in their agencia
            if let Some(manager_agencia_id) = manager_id_entidad {
                if let Some(target_id_entidad) = target_user.id_entidad {
                    if target_id_entidad != manager_agencia_id {
                        return Err(ApplicationError::Forbidden(
                            "Solo puedes gestionar usuarios de tu agencia".to_string()
                        ));
                    }
                }
            }
            Ok(())
        },
        
        _ => Err(ApplicationError::Forbidden("No tienes permisos para gestionar usuarios".to_string())),
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
        UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerenteCadena | UserRole::HotelesGerente | UserRole::AgenciasGerente
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
        &state,
        &auth.user.role,
        auth.user.id_entidad,
        &request.role,
        request.id_entidad,
    ).await?;
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let result = state.container.user_service
        .create_user(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(result.user))
}
