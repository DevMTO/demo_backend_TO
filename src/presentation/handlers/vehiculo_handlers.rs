use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateVehiculoRequest, UpdateVehiculoRequest, VehiculoResponse};
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_vehiculos(
    State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.vehiculo_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(VehiculoResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_vehiculo(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let v = state.container.vehiculo_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;
    Ok(json_ok(VehiculoResponse::from(v)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_vehiculo(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateVehiculoRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.vehiculo_repository.exists_by_placa(&request.placa).await? {
        return Err(ApplicationError::Conflict(format!("Placa {} ya existe", request.placa)));
    }
    let created = state.container.vehiculo_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Vehículo creado: {} (ID: {})", created.placa, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Vehiculo>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Vehiculo,
        created.id,
        &created.placa,
        Some(&created),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo vehículo creado",
        &format!("{} ha creado el vehículo con placa '{}'", auth.user.username, created.placa),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(VehiculoResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_vehiculo(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateVehiculoRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_v = state.container.vehiculo_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;
    let result = state.container.vehiculo_repository.update(&request.apply_to(old_v.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Vehiculo>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Vehiculo,
        result.id,
        Some(&old_v),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Vehículo actualizado",
        &format!("{} ha actualizado el vehículo con placa '{}'", auth.user.username, result.placa),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(VehiculoResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_vehiculo(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    // Get vehiculo info before deleting
    let vehiculo = state.container.vehiculo_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)))?;

    if !state.container.vehiculo_repository.delete(id).await? {
        return Err(ApplicationError::NotFound(format!("Vehículo {} no encontrado", id)));
    }

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Vehiculo>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Vehiculo,
        id,
        Some(&vehiculo),
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Vehículo eliminado",
        &format!("{} ha eliminado el vehículo con placa '{}'", auth.user.username, vehiculo.placa),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_deleted())
}

#[instrument(skip(state, _auth))]
pub async fn list_vehiculos_by_transporte(State(state): State<AppState>, _auth: AuthUser, Path(transporte_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = state.container.vehiculo_repository.find_by_transporte(transporte_id).await?;
    Ok(json_ok(vehiculos.into_iter().map(VehiculoResponse::from).collect::<Vec<_>>()))
}

#[instrument(skip(state, _auth))]
pub async fn list_vehiculos_available(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = state.container.vehiculo_repository.list_available().await?;
    Ok(json_ok(vehiculos.into_iter().map(VehiculoResponse::from).collect::<Vec<_>>()))
}
