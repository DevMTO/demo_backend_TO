//! GET handlers para Tour

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::instrument;
use bigdecimal::BigDecimal;

use crate::application::dtos::TourResponse;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{
    PaginationParams, PaginatedResponse, PaginationInfo, json_ok,
};

#[derive(Debug, Deserialize)]
pub struct TourSearchQuery {
    pub nombre: Option<String>,
    pub min_precio: Option<f64>,
    pub max_precio: Option<f64>,
    pub duracion: Option<i32>,
}

/// Listar tours con paginación (incluye activos e inactivos)
#[instrument(skip(state, _auth))]
pub async fn list_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let options = params.to_options();
    let (items, total, total_pages) = state.container.tour_service
        .list_tours(options, true)
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

/// Buscar tours
#[instrument(skip(state, _auth))]
pub async fn search_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TourSearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let response = if let Some(nombre) = &query.nombre {
        state.container.tour_service.search_tours(nombre).await?
    } else if let (Some(min), Some(max)) = (query.min_precio, query.max_precio) {
        let min_bd = BigDecimal::try_from(min).unwrap_or_default();
        let max_bd = BigDecimal::try_from(max).unwrap_or_default();
        let tours = state.container.tour_repository.find_by_precio_range(min_bd, max_bd).await?;
        tours.into_iter().map(TourResponse::from).collect()
    } else if let Some(duracion) = query.duracion {
        let tours = state.container.tour_repository.find_by_duracion(duracion).await?;
        tours.into_iter().map(TourResponse::from).collect()
    } else {
        let tours = state.container.tour_repository.list(50, 0).await?;
        tours.into_iter().map(TourResponse::from).collect()
    };
    
    Ok(json_ok(response))
}
