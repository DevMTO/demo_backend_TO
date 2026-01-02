use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateGuiaRequest, UpdateGuiaRequest, GuiaResponse};
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_guias(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    
    let (items, total) = state.container.guia_repository.list_with_persona(page_size, offset).await?;
    let total_pages = (total + page_size - 1) / page_size;
    
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { page, page_size, total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_guia(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let g = state.container.guia_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))?;
    Ok(json_ok(GuiaResponse::from(g)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_guia(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateGuiaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.guia_repository.exists_by_carnet(&request.nro_carnet).await? {
        return Err(ApplicationError::Conflict(format!("Carnet {} ya existe", request.nro_carnet)));
    }
    let created = state.container.guia_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Guía creado: {} (ID: {})", created.nro_carnet, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Guia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Guia,
        created.id,
        &created.nro_carnet,
        Some(&created),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo guía creado",
        &format!("{} ha creado el guía con carnet '{}'", auth.user.username, created.nro_carnet),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(GuiaResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_guia(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateGuiaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_g = state.container.guia_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))?;
    let result = state.container.guia_repository.update(&request.apply_to(old_g.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Guia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Guia,
        result.id,
        Some(&old_g),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Guía actualizado",
        &format!("{} ha actualizado el guía con carnet '{}'", auth.user.username, result.nro_carnet),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(GuiaResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_guia(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    // Get guia info before deleting
    let guia = state.container.guia_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", id)))?;

    if !state.container.guia_repository.delete(id).await? { 
        return Err(ApplicationError::NotFound(format!("Guía {} no encontrado", id))); 
    }

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Guia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Guia,
        id,
        Some(&guia),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Guía eliminado",
        &format!("{} ha eliminado el guía con carnet '{}'", auth.user.username, guia.nro_carnet),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_deleted())
}

#[derive(Debug, serde::Deserialize)]
pub struct GuiaSearchQuery { pub idioma: Option<String>, pub especialidad: Option<String> }

#[instrument(skip(state, _auth))]
pub async fn search_guias(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<GuiaSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let guias = if let Some(idioma) = query.idioma {
        state.container.guia_repository.find_by_idioma(&idioma).await?
    } else if let Some(especialidad) = query.especialidad {
        state.container.guia_repository.find_by_especialidad(&especialidad).await?
    } else {
        state.container.guia_repository.list_available().await?
    };
    Ok(json_ok(guias.into_iter().map(GuiaResponse::from).collect::<Vec<_>>()))
}

#[instrument(skip(state, _auth))]
pub async fn list_guias_available(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    let guias = state.container.guia_repository.list_available().await?;
    Ok(json_ok(guias.into_iter().map(GuiaResponse::from).collect::<Vec<_>>()))
}
