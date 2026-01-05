use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::application::dtos::{
    CreateAgenciaRequest, UpdateAgenciaRequest, UpdateAgenciaInterfazRequest, 
    AgenciaResponse, AgenciaListItemDto,
};

use crate::domain::entities::{
    Agencia, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_message,
};

#[instrument(skip(state, auth))]
pub async fn list_agencias(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden listar todas las agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar agencias".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    let limit = page_size;
    
    let (items, total) = state.container.agencia_repository
        .list_with_encargado(limit, offset)
        .await?;
    
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    let response: PaginatedResponse<AgenciaListItemDto> = PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page,
            page_size,
            total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth))]
pub async fn get_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden obtener cualquier agencia por ID
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para ver esta agencia".to_string()));
    }
    
    let agencia = state.container.agencia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}

/// Obtener la agencia del usuario actual
/// 
/// Busca primero por id_entidad si el usuario es de una agencia,
/// o por encargado si el usuario es el responsable de una agencia.
#[instrument(skip(state, auth))]
pub async fn get_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🏢 Buscando agencia para usuario '{}' (id_persona: {:?}, id_entidad: {:?}, role: {:?})", 
        auth.user.username, auth.user.id_persona, auth.user.id_entidad, auth.user.role);
    
    let mut agencia: Option<Agencia> = None;
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth.user.role == UserRole::Agencias;
    
    // Primero intentar por id_entidad si el usuario está relacionado con una agencia
    if is_agencia_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            agencia = state.container.agencia_repository
                .find_by_id(id_entidad)
                .await?;
            if agencia.is_some() {
                info!("✅ Agencia encontrada por id_entidad: {}", id_entidad);
            }
        }
    }
    
    // Si no se encontró, buscar por encargado (id_persona)
    if agencia.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            agencia = state.container.agencia_repository
                .find_by_encargado(persona_id)
                .await?;
            if agencia.is_some() {
                info!("✅ Agencia encontrada por encargado (persona_id: {})", persona_id);
            }
        }
    }
    
    match agencia {
        Some(a) => {
            let response: AgenciaResponse = a.into();
            Ok(json_ok(response))
        }
        None => {
            info!("ℹ️ Usuario '{}' no tiene agencia asociada", auth.user.username);
            Err(ApplicationError::NotFound("No tienes una agencia asociada".to_string()))
        }
    }
}

/// Actualizar mi propia agencia
/// 
/// Permite a usuarios de tipo Agencia o encargados actualizar su agencia.
/// Solo pueden actualizar: paleta_colores, direccion, telefono.
/// No pueden modificar: ruc, nombre, encargado, estado.
#[instrument(skip(state, auth, request))]
pub async fn update_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🏢 Usuario '{}' intenta actualizar su agencia", auth.user.username);
    
    // Buscar la agencia del usuario
    let mut agencia: Option<Agencia> = None;
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth.user.role == UserRole::Agencias;
    
    // Intentar por id_entidad si el usuario está relacionado con una agencia
    if is_agencia_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            agencia = state.container.agencia_repository
                .find_by_id(id_entidad)
                .await?;
        }
    }
    
    // Si no se encontró, buscar por encargado (id_persona)
    if agencia.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            agencia = state.container.agencia_repository
                .find_by_encargado(persona_id)
                .await?;
        }
    }
    
    let agencia = agencia.ok_or_else(|| {
        ApplicationError::NotFound("No tienes una agencia asociada".to_string())
    })?;
    
    let agencia_id = agencia.id;
    
    // Crear un request limitado: solo permitir ciertos campos
    let limited_request = UpdateAgenciaRequest {
        nombre: None, // No puede cambiar nombre
        ruc: None, // No puede cambiar RUC
        direccion: request.direccion,
        telefono: request.telefono,
        correo: request.correo,
        encargado: None, // No puede cambiar encargado
        paleta_colores: request.paleta_colores,
        media: request.media, // Puede actualizar media (logo, etc.)
        is_active: None, // No puede cambiar estado
    };
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_agencia_use_case
        .execute(agencia_id, limited_request, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        response.id,
        None::<&Agencia>,
        None::<&Agencia>,
        Some(vec!["paleta_colores".to_string(), "direccion".to_string(), "telefono".to_string()]),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de actualización de agencia: {}", e);
    }
    
    info!("✅ Agencia '{}' actualizada por su encargado/usuario {}", response.nombre, auth.user.username);
    
    Ok(json_ok(response))
}

/// Actualizar solo la interfaz de mi agencia (logo y paleta de colores)
/// 
/// Endpoint PATCH que permite actualizar solo logo y paleta_colores.
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_agencia_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🎨 Usuario '{}' actualiza interfaz de su agencia", auth.user.username);
    
    // Buscar la agencia del usuario
    let mut agencia: Option<Agencia> = None;
    
    // Verificar si el usuario tiene rol de agencia
    let is_agencia_user = auth.user.role == UserRole::Agencias;
    
    if is_agencia_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            agencia = state.container.agencia_repository
                .find_by_id(id_entidad)
                .await?;
        }
    }
    
    if agencia.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            agencia = state.container.agencia_repository
                .find_by_encargado(persona_id)
                .await?;
        }
    }
    
    let old_agencia = agencia.ok_or_else(|| {
        ApplicationError::NotFound("No tienes una agencia asociada".to_string())
    })?;
    
    let agencia_id = old_agencia.id;
    
    // Aplicar cambios solo de interfaz
    let updated = request.apply_to(old_agencia.clone(), Some(auth.user.id));
    let result = state.container.agencia_repository.update(&updated).await?;
    
    // Logging
    let _ = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        agencia_id,
        Some(&old_agencia),
        Some(&result),
        Some(vec!["paleta_colores".to_string(), "media".to_string()]),
        None,
    ).await;
    
    info!("✅ Interfaz de agencia '{}' actualizada", result.nombre);
    
    Ok(json_ok(AgenciaResponse::from(result)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden crear agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para crear la agencia
    let response = state.container.create_agencia_use_case
        .execute(request, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_create::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        response.id,
        &response.nombre,
        None::<&Agencia>,
        None,
    ).await {
        warn!("⚠️ Error al registrar log de creación de agencia: {}", e);
    }
    
    // Notificación a admins con broadcast SSE
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nueva agencia creada",
        &format!("Se ha creado la agencia '{}' (RUC: {})", response.nombre, response.ruc),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de agencia creada: {}", e);
    }
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden actualizar agencias por ID
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para actualizar agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_agencia_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        response.id,
        None::<&Agencia>,
        None::<&Agencia>,
        None,
        None,
    ).await {
        warn!("⚠️ Error al registrar log de actualización de agencia: {}", e);
    }
    
    // Notificación a admins con broadcast SSE
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Agencia actualizada",
        &format!("La agencia '{}' ha sido actualizada por {}", response.nombre, auth.user.username),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de agencia actualizada: {}", e);
    }
    
    info!("✅ Agencia actualizada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth))]
pub async fn delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden desactivar agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar agencias".to_string()));
    }
    
    // Obtener agencia antes de desactivar
    let agencia = state.container.agencia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
    
    // Usar el caso de uso para desactivar
    state.container.deactivate_agencia_use_case
        .execute(id, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_delete::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        id,
        Some(&agencia),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de desactivación de agencia: {}", e);
    }
    
    // Notificación a admins con broadcast SSE
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Agencia desactivada",
        &format!("La agencia '{}' ha sido desactivada por {}", agencia.nombre, auth.user.username),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de agencia desactivada: {}", e);
    }
    
    info!("🗑️ Agencia desactivada: {} (ID: {})", agencia.nombre, id);
    
    Ok(json_message("Agencia desactivada correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden restaurar agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar agencias".to_string()));
    }
    
    // Usar el caso de uso para restaurar
    state.container.restore_agencia_use_case
        .execute(id, auth.user.id)
        .await?;
    
    // Obtener agencia restaurada
    let agencia = state.container.agencia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        id,
        None::<&Agencia>,
        None::<&Agencia>,
        Some(vec!["is_active".to_string()]),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de restauración de agencia: {}", e);
    }
    
    // Notificación a admins con broadcast SSE
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Agencia restaurada",
        &format!("La agencia '{}' ha sido restaurada por {}", agencia.nombre, auth.user.username),
        NotificationType::Success,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de agencia restaurada: {}", e);
    }
    
    info!("✅ Agencia restaurada: {} (ID: {})", agencia.nombre, id);
    
    Ok(json_message("Agencia restaurada correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn get_agencia_by_ruc(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ruc): Path<String>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden buscar agencias por RUC
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para buscar agencias".to_string()));
    }
    
    let agencia = state.container.agencia_repository
        .find_by_ruc(&ruc)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia con RUC {} no encontrada", ruc)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}
