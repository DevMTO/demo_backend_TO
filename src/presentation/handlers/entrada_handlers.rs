use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateEntradaRequest, UpdateEntradaRequest, EntradaResponse};
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};
use crate::domain::errors::ApplicationError;

use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[instrument(skip(state, _auth))]
pub async fn list_entradas(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.entrada_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(EntradaResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_entrada(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let e = state.container.entrada_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;
    Ok(json_ok(EntradaResponse::from(e)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_entrada(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateEntradaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let created = state.container.entrada_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Entrada creada: {} (ID: {})", created.nombre, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Entrada>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Entrada,
        created.id,
        &created.nombre,
        Some(&created),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nueva entrada creada",
        &format!("{} ha creado la entrada '{}'", auth.user.username, created.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(EntradaResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateEntradaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_e = state.container.entrada_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;
    let result = state.container.entrada_repository.update(&request.apply_to(old_e.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Entrada>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Entrada,
        result.id,
        Some(&old_e),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Entrada actualizada",
        &format!("{} ha actualizado la entrada '{}'", auth.user.username, result.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(EntradaResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    // Get entrada info before deleting
    let entrada = state.container.entrada_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;

    if !state.container.entrada_repository.soft_delete(id, auth.user.id).await? { 
        return Err(ApplicationError::NotFound(format!("Entrada {} no encontrada", id))); 
    }
    info!("🗑️ Entrada {} desactivada", id);

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Entrada>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Entrada,
        id,
        Some(&entrada),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Entrada eliminada",
        &format!("{} ha eliminado la entrada '{}'", auth.user.username, entrada.nombre),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Entrada desactivada"))
}

#[instrument(skip(state, auth))]
pub async fn restore_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.entrada_repository.restore(id, auth.user.id).await? { 
        return Err(ApplicationError::NotFound(format!("Entrada {} no encontrada", id))); 
    }

    // Get entrada info after restore
    let entrada = state.container.entrada_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", id)))?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Entrada>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Entrada,
        id,
        None,
        Some(&entrada),
        Some(vec!["is_active".to_string()]),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Entrada restaurada",
        &format!("{} ha restaurado la entrada '{}'", auth.user.username, entrada.nombre),
        NotificationType::Success,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Entrada restaurada"))
}

#[derive(Debug, serde::Deserialize)]
pub struct EntradaSearchQuery { pub tipo: Option<String>, pub ruta: Option<String> }

#[instrument(skip(state, _auth))]
pub async fn search_entradas(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<EntradaSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let entradas = if let Some(tipo) = query.tipo {
        state.container.entrada_repository.find_by_tipo(&tipo).await?
    } else if let Some(ruta) = query.ruta {
        state.container.entrada_repository.find_by_ruta(&ruta).await?
    } else {
        state.container.entrada_repository.list_paginated(Default::default()).await?.data
    };
    Ok(json_ok(entradas.into_iter().map(EntradaResponse::from).collect::<Vec<_>>()))
}
