//! Tour Handlers - Endpoints HTTP para gestión de tours
//!
//! Los handlers solo manejan preocupaciones HTTP:
//! - Validación de requests
//! - Extracción de parámetros
//! - Conversión de respuestas
//!
//! La lógica de negocio está en TourService

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};
use validator::Validate;
use bigdecimal::BigDecimal;

use crate::application::dtos::{
    CreateTourRequest, UpdateTourRequest, TourResponse,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_message,
};

/// Listar tours con paginación
#[instrument(skip(state, _auth))]
pub async fn list_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let options = params.to_options();
    let (items, total, total_pages) = state.container.tour_service
        .list_tours(options, false)
        .await?;
    
    let response: PaginatedResponse<TourResponse> = PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page: params.page,
            page_size: params.page_size,
            total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

/// Obtener tour por ID
#[instrument(skip(state, _auth))]
pub async fn get_tour(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.tour_service
        .get_tour(id)
        .await?;
    
    Ok(json_ok(response))
}

/// Crear nuevo tour
#[instrument(skip(state, auth, request))]
pub async fn create_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio (incluye logging)
    let response = state.container.tour_service
        .create_tour(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("✅ Tour creado: {} (ID: {})", response.nombre, response.id);
    Ok(json_created(response))
}

/// Actualizar tour existente
#[instrument(skip(state, auth, request))]
pub async fn update_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio (incluye logging)
    let response = state.container.tour_service
        .update_tour(
            id,
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("✅ Tour actualizado: {} (ID: {})", response.nombre, response.id);
    Ok(json_ok(response))
}

/// Desactivar tour (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio (incluye logging)
    state.container.tour_service
        .deactivate_tour(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("🗑️ Tour desactivado: ID {}", id);
    Ok(json_message("Tour desactivado correctamente"))
}

/// Restaurar tour desactivado
#[instrument(skip(state, auth))]
pub async fn restore_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio (incluye logging)
    state.container.tour_service
        .restore_tour(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("✅ Tour restaurado: ID {}", id);
    Ok(json_message("Tour restaurado correctamente"))
}

#[derive(Debug, serde::Deserialize)]
pub struct TourSearchQuery {
    pub nombre: Option<String>,
    pub min_precio: Option<f64>,
    pub max_precio: Option<f64>,
    pub duracion: Option<i32>,
}

/// Buscar tours
#[instrument(skip(state, _auth))]
pub async fn search_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TourSearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Buscar por nombre si se proporciona
    let response = if let Some(nombre) = &query.nombre {
        state.container.tour_service.search_tours(nombre).await?
    } else if let (Some(min), Some(max)) = (query.min_precio, query.max_precio) {
        // Para rango de precios, usar repositorio directamente (método específico)
        let min_bd = BigDecimal::try_from(min).unwrap_or_default();
        let max_bd = BigDecimal::try_from(max).unwrap_or_default();
        let tours = state.container.tour_repository.find_by_precio_range(min_bd, max_bd).await?;
        tours.into_iter().map(TourResponse::from).collect()
    } else if let Some(duracion) = query.duracion {
        // Buscar por duración usando repositorio
        let tours = state.container.tour_repository.find_by_duracion(duracion).await?;
        tours.into_iter().map(TourResponse::from).collect()
    } else {
        // Lista por defecto
        let tours = state.container.tour_repository.list(50, 0).await?;
        tours.into_iter().map(TourResponse::from).collect()
    };
    
    Ok(json_ok(response))
}
