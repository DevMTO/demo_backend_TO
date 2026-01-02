use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateConductorRequest, UpdateConductorRequest, ConductorResponse};
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_conductores(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.conductor_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(ConductorResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_conductor(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let c = state.container.conductor_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
    Ok(json_ok(ConductorResponse::from(c)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_conductor(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateConductorRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.conductor_repository.exists_by_brevete(&request.nro_brevete).await? {
        return Err(ApplicationError::Conflict(format!("Brevete {} ya existe", request.nro_brevete)));
    }
    let created = state.container.conductor_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Conductor creado: {} (ID: {})", created.nro_brevete, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Conductor>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Conductor,
        created.id,
        &created.nro_brevete,
        Some(&created),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo conductor creado",
        &format!("{} ha creado el conductor con brevete '{}'", auth.user.username, created.nro_brevete),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(ConductorResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_conductor(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateConductorRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_c = state.container.conductor_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
    let result = state.container.conductor_repository.update(&request.apply_to(old_c.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Conductor>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Conductor,
        result.id,
        Some(&old_c),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Conductor actualizado",
        &format!("{} ha actualizado el conductor con brevete '{}'", auth.user.username, result.nro_brevete),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(ConductorResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_conductor(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    // Get conductor info before deleting
    let conductor = state.container.conductor_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;

    if !state.container.conductor_repository.delete(id).await? { 
        return Err(ApplicationError::NotFound(format!("Conductor {} no encontrado", id))); 
    }

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Conductor>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Conductor,
        id,
        Some(&conductor),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Conductor eliminado",
        &format!("{} ha eliminado el conductor con brevete '{}'", auth.user.username, conductor.nro_brevete),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_deleted())
}

#[instrument(skip(state, _auth))]
pub async fn list_conductores_by_transporte(State(state): State<AppState>, _auth: AuthUser, Path(transporte_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_repository.find_by_transporte(transporte_id).await?;
    Ok(json_ok(conductores.into_iter().map(ConductorResponse::from).collect::<Vec<_>>()))
}

#[instrument(skip(state, _auth))]
pub async fn list_conductores_available(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_repository.list_available().await?;
    Ok(json_ok(conductores.into_iter().map(ConductorResponse::from).collect::<Vec<_>>()))
}
