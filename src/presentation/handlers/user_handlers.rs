use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, instrument};
use validator::Validate;

use crate::domain::entities::{User, UserRole, UserStatus, Persona, TipoDocumento};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted, create_paginated_response};
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{CreateUserRequest, UpdateUserRequest, UserDetailDto};

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
    info!("📋 Listando usuarios (page: {}, size: {})", params.page, params.page_size);
    
    let page_size = params.page_size.min(100).max(1);
    let offset = (params.page - 1).max(0) * page_size;
    
    let (users, total) = state.container.user_repository
        .list_users_with_details(page_size, offset)
        .await?;
    
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
    info!("🔍 Buscando usuario: {}", id);
    
    let user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    Ok(json_ok(UserDetailDto::from(user)))
}

/// Crear un nuevo usuario (opcionalmente con persona nueva)
#[instrument(skip(state, auth, request))]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar si email ya existe
    if state.container.user_repository.exists_by_email(&request.email).await? {
        return Err(ApplicationError::Conflict(format!("Email {} ya registrado", request.email)));
    }
    
    // Verificar si username ya existe
    if state.container.user_repository.exists_by_username(&request.username).await? {
        return Err(ApplicationError::Conflict(format!("Username {} ya existe", request.username)));
    }
    
    // Determinar id_persona
    let id_persona = if let Some(id) = request.id_persona {
        // Verificar que la persona existe
        let _persona = state.container.persona_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona {} no encontrada", id)))?;
        Some(id)
    } else if let Some(nueva_persona) = request.nueva_persona {
        // Validar datos de nueva persona
        nueva_persona.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
        
        // Verificar que el documento no exista
        if state.container.persona_repository.exists_by_documento(&nueva_persona.tipo_documento, &nueva_persona.nro_documento).await? {
            return Err(ApplicationError::Conflict(format!("Documento {} ya registrado", nueva_persona.nro_documento)));
        }
        
        // Crear la persona
        let now = chrono::Utc::now();
        let tipo_doc = nueva_persona.tipo_documento.parse::<TipoDocumento>()
            .unwrap_or(TipoDocumento::Dni);
        
        let persona = Persona {
            id: 0,
            tipo_documento: tipo_doc,
            nro_documento: nueva_persona.nro_documento,
            nombre: nueva_persona.nombre,
            apellidos: nueva_persona.apellidos,
            telefono: nueva_persona.telefono,
            correo: Some(request.email.clone()), // Usar el email del usuario
            fecha_nacimiento: nueva_persona.fecha_nacimiento,
            created_at: now,
            updated_at: now,
            created_by: Some(auth.user.id),
            updated_by: Some(auth.user.id),
        };
        
        let created_persona = state.container.persona_repository.create(&persona).await?;
        info!("✅ Persona creada: {} {} (ID: {})", created_persona.nombre, created_persona.apellidos, created_persona.id);
        Some(created_persona.id)
    } else {
        None // Usuario sin persona asociada
    };
    
    // Hash de la contraseña
    let password_hash = state.container.password_hasher.hash(&request.password)?;
    
    // Parsear el rol
    let role = request.role.parse::<UserRole>()
        .map_err(|_| ApplicationError::Validation(format!("Rol inválido: {}", request.role)))?;
    
    // Crear la entidad User
    let now = chrono::Utc::now();
    let new_user = User {
        id: 0,
        id_persona,
        username: request.username.clone(),
        email: request.email.to_lowercase(),
        password_hash,
        role,
        id_entidad: request.id_entidad,
        nombre_entidad: request.nombre_entidad,
        status: UserStatus::Activo,
        last_login: None,
        created_at: now,
        updated_at: now,
        created_by: Some(auth.user.id),
        updated_by: Some(auth.user.id),
    };
    
    let created = state.container.user_repository.create(&new_user).await?;
    info!("✅ Usuario creado: {} (ID: {})", created.username, created.id);
    
    Ok(json_created(UserDetailDto::from(created)))
}

/// Actualizar un usuario existente
#[instrument(skip(state, auth, request))]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Buscar usuario existente
    let user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // Verificar que no se esté duplicando el email si se está cambiando
    if let Some(ref new_email) = request.email {
        let email_lower = new_email.to_lowercase();
        if email_lower != user.email {
            if state.container.user_repository.exists_by_email(&email_lower).await? {
                return Err(ApplicationError::Conflict(format!("Email {} ya registrado", new_email)));
            }
        }
    }
    
    // Aplicar cambios
    let updated = request.apply_to(user, Some(auth.user.id));
    let result = state.container.user_repository.update(&updated).await?;
    
    info!("✅ Usuario actualizado: {} (ID: {})", result.username, result.id);
    Ok(json_ok(UserDetailDto::from(result)))
}

/// Eliminar (desactivar) un usuario
#[instrument(skip(state, _auth))]
pub async fn delete_user(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe
    let user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // No permitir eliminar el propio usuario
    // (Comentado porque puede ser válido en algunos casos)
    // if user.id == auth.user.id {
    //     return Err(ApplicationError::Forbidden("No puedes eliminar tu propio usuario".to_string()));
    // }
    
    state.container.user_repository.delete(id).await?;
    
    info!("🗑️ Usuario desactivado: {} (ID: {})", user.username, id);
    Ok(json_deleted())
}
