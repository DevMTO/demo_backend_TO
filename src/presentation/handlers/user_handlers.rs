use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::domain::entities::{
    User, UserRole, UserStatus, Persona, TipoDocumento, EntityType,
    NotificationType, NotificationCategory, NotificationPriority,
};
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
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_create::<User>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::User,
        created.id,
        &created.username,
        Some(&created),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de creación de usuario: {}", e);
    }
    
    // Notificación a admins
    if let Err(e) = state.container.notification_service.notify_roles(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo usuario creado",
        &format!("Se ha creado el usuario '{}' con rol {}", created.username, created.role),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de usuario creado: {}", e);
    }
    
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
    let old_user = user.clone();
    let updated = request.apply_to(user, Some(auth.user.id));
    let result = state.container.user_repository.update(&updated).await?;
    
    info!("✅ Usuario actualizado: {} (ID: {})", result.username, result.id);
    
    // Detectar campos cambiados
    let mut changed_fields = Vec::new();
    if old_user.email != result.email { changed_fields.push("email".to_string()); }
    if old_user.role != result.role { changed_fields.push("role".to_string()); }
    if old_user.status != result.status { changed_fields.push("status".to_string()); }
    if old_user.id_entidad != result.id_entidad { changed_fields.push("id_entidad".to_string()); }
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<User>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::User,
        result.id,
        Some(&old_user),
        Some(&result),
        if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
        None,
    ).await {
        warn!("⚠️ Error al registrar log de actualización de usuario: {}", e);
    }
    
    // Notificación al usuario afectado si fue actualizado por otro
    if result.id != auth.user.id {
        let notification_msg = if changed_fields.is_empty() {
            "Tu cuenta ha sido actualizada".to_string()
        } else {
            format!("Se actualizaron los siguientes campos de tu cuenta: {}", changed_fields.join(", "))
        };
        
        if let Err(e) = state.container.notification_service.notify_user(
            result.id,
            "Cuenta actualizada",
            &notification_msg,
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(auth.user.id),
        ).await {
            warn!("⚠️ Error al enviar notificación de actualización: {}", e);
        }
    }
    
    Ok(json_ok(UserDetailDto::from(result)))
}

/// Eliminar (desactivar) un usuario
#[instrument(skip(state, auth))]
pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe
    let user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // No permitir eliminar el propio usuario
    if user.id == auth.user.id {
        return Err(ApplicationError::Forbidden("No puedes desactivar tu propio usuario".to_string()));
    }
    
    state.container.user_repository.delete(id).await?;
    
    info!("🗑️ Usuario desactivado: {} (ID: {})", user.username, id);
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_delete::<User>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::User,
        id,
        Some(&user),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de desactivación de usuario: {}", e);
    }
    
    // Notificación al usuario desactivado
    if let Err(e) = state.container.notification_service.notify_user(
        id,
        "Cuenta desactivada",
        "Tu cuenta ha sido desactivada por un administrador. Contacta con soporte si crees que es un error.",
        NotificationType::Warning,
        NotificationCategory::Auth,
        NotificationPriority::High,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de desactivación: {}", e);
    }
    
    // Notificación a admins
    if let Err(e) = state.container.notification_service.notify_roles(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Usuario desactivado",
        &format!("El usuario '{}' ha sido desactivado por {}", user.username, auth.user.username),
        NotificationType::Warning,
        NotificationCategory::Auth,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación a admins: {}", e);
    }
    
    Ok(json_deleted())
}
/// Activar un usuario
#[instrument(skip(state, auth))]
pub async fn activate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe
    let mut user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // Verificar que no está ya activo
    if user.status == UserStatus::Activo {
        return Err(ApplicationError::Conflict("El usuario ya está activo".to_string()));
    }
    
    let old_status = user.status.clone();
    user.status = UserStatus::Activo;
    user.updated_at = chrono::Utc::now();
    user.updated_by = Some(auth.user.id);
    
    let result = state.container.user_repository.update(&user).await?;
    
    info!("✅ Usuario activado: {} (ID: {})", result.username, id);
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<User>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::User,
        id,
        None::<&User>,
        None::<&User>,
        Some(vec!["status".to_string()]),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de activación de usuario: {}", e);
    }
    
    // Notificación al usuario activado
    if let Err(e) = state.container.notification_service.notify_user(
        id,
        "Cuenta activada",
        "Tu cuenta ha sido activada nuevamente. Ya puedes iniciar sesión.",
        NotificationType::Success,
        NotificationCategory::Auth,
        NotificationPriority::High,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de activación: {}", e);
    }
    
    // Notificación a admins
    if let Err(e) = state.container.notification_service.notify_roles(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Usuario activado",
        &format!("El usuario '{}' ha sido activado por {} (estado anterior: {:?})", result.username, auth.user.username, old_status),
        NotificationType::Info,
        NotificationCategory::Auth,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación a admins: {}", e);
    }
    
    Ok(json_ok(UserDetailDto::from(result)))
}

/// Desactivar un usuario
#[instrument(skip(state, auth))]
pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe
    let mut user = state.container.user_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
    
    // No permitir desactivar el propio usuario
    if user.id == auth.user.id {
        return Err(ApplicationError::Forbidden("No puedes desactivar tu propio usuario".to_string()));
    }
    
    // Verificar que no está ya inactivo
    if user.status == UserStatus::Inactivo {
        return Err(ApplicationError::Conflict("El usuario ya está desactivado".to_string()));
    }
    
    let old_status = user.status.clone();
    user.status = UserStatus::Inactivo;
    user.updated_at = chrono::Utc::now();
    user.updated_by = Some(auth.user.id);
    
    let result = state.container.user_repository.update(&user).await?;
    
    info!("🔒 Usuario desactivado manualmente: {} (ID: {})", result.username, id);
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<User>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::User,
        id,
        None::<&User>,
        None::<&User>,
        Some(vec!["status".to_string()]),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de desactivación de usuario: {}", e);
    }
    
    // Notificación al usuario desactivado
    if let Err(e) = state.container.notification_service.notify_user(
        id,
        "Cuenta desactivada",
        "Tu cuenta ha sido desactivada por un administrador. Contacta con soporte si crees que es un error.",
        NotificationType::Warning,
        NotificationCategory::Auth,
        NotificationPriority::High,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de desactivación: {}", e);
    }
    
    // Notificación a admins
    if let Err(e) = state.container.notification_service.notify_roles(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Usuario desactivado manualmente",
        &format!("El usuario '{}' ha sido desactivado manualmente por {} (estado anterior: {:?})", result.username, auth.user.username, old_status),
        NotificationType::Warning,
        NotificationCategory::Auth,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación a admins: {}", e);
    }
    
    Ok(json_ok(UserDetailDto::from(result)))
}
