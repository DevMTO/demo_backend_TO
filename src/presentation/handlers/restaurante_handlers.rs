use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateRestauranteRequest, UpdateRestauranteRequest, RestauranteResponse};
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[instrument(skip(state, _auth))]
pub async fn list_restaurantes(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let limit = params.page_size.min(100);
    let offset = (params.page - 1).max(0) * limit;
    let (items, total) = state.container.restaurante_repository.list_with_encargado(limit, offset).await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { page: params.page, page_size: limit, total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_restaurante(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    Ok(json_ok(RestauranteResponse::from(r)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_restaurante(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateRestauranteRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let created = state.container.restaurante_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Restaurante creado: {} (ID: {})", created.nombre, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Restaurante>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Restaurante,
        created.id,
        &created.nombre,
        Some(&created),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo restaurante creado",
        &format!("{} ha creado el restaurante '{}'", auth.user.username, created.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(RestauranteResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateRestauranteRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let result = state.container.restaurante_repository.update(&request.apply_to(old_r.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Restaurante>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Restaurante,
        result.id,
        Some(&old_r),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Restaurante actualizado",
        &format!("{} ha actualizado el restaurante '{}'", auth.user.username, result.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(RestauranteResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let mut updated = r.clone();
    updated.is_active = false;
    updated.updated_by = Some(auth.user.id);
    updated.updated_at = chrono::Utc::now();
    state.container.restaurante_repository.update(&updated).await?;

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Restaurante>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Restaurante,
        id,
        Some(&r),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Restaurante eliminado",
        &format!("{} ha eliminado el restaurante '{}'", auth.user.username, r.nombre),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Restaurante desactivado correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let mut updated = r.clone();
    updated.is_active = true;
    updated.updated_by = Some(auth.user.id);
    updated.updated_at = chrono::Utc::now();
    let restored = state.container.restaurante_repository.update(&updated).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Restaurante>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Restaurante,
        id,
        Some(&r),
        Some(&restored),
        Some(vec!["is_active".to_string()]),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Restaurante restaurado",
        &format!("{} ha restaurado el restaurante '{}'", auth.user.username, r.nombre),
        NotificationType::Success,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Restaurante restaurado correctamente"))
}

#[derive(Debug, serde::Deserialize)]
pub struct RestauranteSearchQuery { pub tipo_atencion: Option<String>, pub min_capacidad: Option<i32> }

#[instrument(skip(state, _auth))]
pub async fn search_restaurantes(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<RestauranteSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let result = if let Some(tipo) = query.tipo_atencion {
        state.container.restaurante_repository.find_by_tipo_atencion(&tipo).await?
    } else if let Some(min_cap) = query.min_capacidad {
        state.container.restaurante_repository.find_by_min_capacity(min_cap).await?
    } else {
        let paginated = state.container.restaurante_repository.list_paginated(crate::application::ports::PaginationOptions { limit: Some(100), offset: Some(0) }).await?;
        paginated.data
    };
    Ok(json_ok(result.into_iter().map(RestauranteResponse::from).collect::<Vec<_>>()))
}
