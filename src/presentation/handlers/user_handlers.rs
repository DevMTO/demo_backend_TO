use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::instrument;
use validator::Validate;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted, create_paginated_response};
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{CreateUserRequest, UpdateUserRequest, AdminChangePasswordRequest};

#[derive(Debug, Deserialize)]
pub struct ListUsersParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

/// Listar usuarios con paginación
#[instrument(skip(state, _auth))]
pub async fn list_users(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListUsersParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let (users, total) = state.container.user_service
        .list_users(params.page, params.page_size)
        .await?;
    
    let page_size = params.page_size.min(100).max(1);
    let response = create_paginated_response(users, total, params.page, page_size);
    
    Ok(json_ok(response))
}

/// Obtener un usuario por ID
#[instrument(skip(state, _auth))]
pub async fn get_user(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let user = state.container.user_service
        .get_user(id)
        .await?;
    
    Ok(json_ok(user))
}

/// Crear un nuevo usuario (opcionalmente con persona nueva)
/// Solo SuperAdmin puede crear usuarios
#[instrument(skip(state, auth, request))]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede crear usuarios".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let result = state.container.user_service
        .create_user(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(result.user))
}

/// Actualizar un usuario existente
/// Solo SuperAdmin puede editar usuarios
#[instrument(skip(state, auth, request))]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede editar usuarios".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let result = state.container.user_service
        .update_user(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(result.user))
}

/// Eliminar (desactivar) un usuario
/// Solo SuperAdmin puede eliminar usuarios
#[instrument(skip(state, auth))]
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar usuarios".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let _user = state.container.user_service
        .delete_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}

/// Eliminación permanente de un usuario (hard delete)
/// Solo SuperAdmin puede ejecutar esta acción
#[instrument(skip(state, auth))]
pub async fn hard_delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente usuarios".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let _user = state.container.user_service
        .hard_delete_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_deleted())
}

/// Activar un usuario
/// Solo SuperAdmin puede activar usuarios
#[instrument(skip(state, auth))]
pub async fn activate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede activar usuarios".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let (user_dto, _old_active) = state.container.user_service
        .activate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}

/// Desactivar un usuario
/// Solo SuperAdmin puede desactivar usuarios
#[instrument(skip(state, auth))]
pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede desactivar usuarios".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let (user_dto, _old_active) = state.container.user_service
        .deactivate_user(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}

/// Cambiar contraseña de un usuario (solo SuperAdmin)
#[instrument(skip(state, auth, request))]
pub async fn admin_change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<AdminChangePasswordRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede cambiar contraseñas de otros usuarios".to_string()));
    }
    
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let user_dto = state.container.user_service
        .admin_change_password(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(user_dto))
}
