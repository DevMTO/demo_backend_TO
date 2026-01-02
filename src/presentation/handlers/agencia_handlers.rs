use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::application::dtos::{
    CreateAgenciaRequest, UpdateAgenciaRequest, AgenciaResponse, AgenciaListItemDto,
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

#[instrument(skip(state, _auth))]
pub async fn list_agencias(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
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

#[instrument(skip(state, _auth))]
pub async fn get_agencia(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let agencia = state.container.agencia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}

#[instrument(skip(state, auth, request))]
pub async fn create_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
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

#[instrument(skip(state, _auth))]
pub async fn get_agencia_by_ruc(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(ruc): Path<String>,
) -> Result<impl IntoResponse, ApplicationError> {
    let agencia = state.container.agencia_repository
        .find_by_ruc(&ruc)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia con RUC {} no encontrada", ruc)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}
