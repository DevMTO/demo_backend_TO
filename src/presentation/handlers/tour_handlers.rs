use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, warn, instrument};
use validator::Validate;
use bigdecimal::BigDecimal;

use crate::application::dtos::{
    CreateTourRequest, UpdateTourRequest, TourResponse,
};

use crate::domain::entities::{
    Tour, EntityType, UserRole,
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
pub async fn list_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let options = params.to_options();
    let result = state.container.tour_repository
        .list_paginated(options)
        .await?;
    
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    let response: PaginatedResponse<TourResponse> = PaginatedResponse {
        items: result.data.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo {
            page,
            page_size,
            total: result.total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn get_tour(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let tour = state.container.tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
    
    let response: TourResponse = tour.into();
    Ok(json_ok(response))
}

#[instrument(skip(state, auth, request))]
pub async fn create_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para crear el tour
    let response = state.container.create_tour_use_case
        .execute(request, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_create::<Tour>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Tour,
        response.id,
        &response.nombre,
        None::<&Tour>,
        None,
    ).await {
        warn!("⚠️ Error al registrar log de creación de tour: {}", e);
    }
    
    // Notificación a admins via SSE broadcast
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo tour creado",
        &format!("Se ha creado el tour '{}' (Duración: {} días)", response.nombre, response.duracion_dias.unwrap_or(0)),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de tour creado: {}", e);
    }
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_tour_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<Tour>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Tour,
        response.id,
        None::<&Tour>,
        None::<&Tour>,
        None,
        None,
    ).await {
        warn!("⚠️ Error al registrar log de actualización de tour: {}", e);
    }
    
    // Notificación a admins via SSE broadcast
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Tour actualizado",
        &format!("El tour '{}' ha sido actualizado por {}", response.nombre, auth.user.username),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de tour actualizado: {}", e);
    }
    
    info!("✅ Tour actualizado: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth))]
pub async fn delete_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Obtener tour antes de desactivar
    let tour = state.container.tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
    
    // Usar el caso de uso para desactivar
    state.container.deactivate_tour_use_case
        .execute(id, auth.user.id)
        .await?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_delete::<Tour>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Tour,
        id,
        Some(&tour),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de desactivación de tour: {}", e);
    }
    
    // Notificación a admins via SSE broadcast
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Tour desactivado",
        &format!("El tour '{}' ha sido desactivado por {}", tour.nombre, auth.user.username),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de tour desactivado: {}", e);
    }
    
    info!("🗑️ Tour desactivado: {} (ID: {})", tour.nombre, id);
    
    Ok(json_message("Tour desactivado correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para restaurar
    state.container.restore_tour_use_case
        .execute(id, auth.user.id)
        .await?;
    
    // Obtener tour restaurado
    let tour = state.container.tour_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Tour {} no encontrado", id)))?;
    
    // Logging del evento
    if let Err(e) = state.container.logging_service.log_update::<Tour>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Tour,
        id,
        None::<&Tour>,
        None::<&Tour>,
        Some(vec!["is_active".to_string()]),
        None,
    ).await {
        warn!("⚠️ Error al registrar log de restauración de tour: {}", e);
    }
    
    // Notificación a admins via SSE broadcast
    if let Err(e) = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Tour restaurado",
        &format!("El tour '{}' ha sido restaurado por {}", tour.nombre, auth.user.username),
        NotificationType::Success,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await {
        warn!("⚠️ Error al enviar notificación de tour restaurado: {}", e);
    }
    
    info!("✅ Tour restaurado: {} (ID: {})", tour.nombre, id);
    
    Ok(json_message("Tour restaurado correctamente"))
}

#[derive(Debug, serde::Deserialize)]
pub struct TourSearchQuery {
    pub nombre: Option<String>,
    pub min_precio: Option<f64>,
    pub max_precio: Option<f64>,
    pub duracion: Option<i32>,
}

#[instrument(skip(state, _auth))]
pub async fn search_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TourSearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Usar los casos de uso para búsquedas
    let response = if let Some(nombre) = &query.nombre {
        state.container.search_tours_use_case.by_name(nombre).await?
    } else if let Some(duracion) = query.duracion {
        state.container.search_tours_use_case.by_duracion(duracion).await?
    } else if let (Some(min), Some(max)) = (query.min_precio, query.max_precio) {
        // Para rango de precios, usar repositorio directamente (método específico)
        let min_bd = BigDecimal::try_from(min).unwrap_or_default();
        let max_bd = BigDecimal::try_from(max).unwrap_or_default();
        let tours = state.container.tour_repository.find_by_precio_range(min_bd, max_bd).await?;
        tours.into_iter().map(TourResponse::from).collect()
    } else {
        // Lista por defecto
        let tours = state.container.tour_repository.list(50, 0).await?;
        tours.into_iter().map(TourResponse::from).collect()
    };
    
    Ok(json_ok(response))
}
