//! GET handlers para Hoteles

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::application::dtos::HotelResponse;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{
    PaginationParams, PaginatedResponse, PaginationInfo, json_ok,
};

/// GET /api/v1/hoteles - Listar todos los hoteles
#[instrument(skip(state, auth))]
pub async fn list_hoteles(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar hoteles".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    
    let (items, total) = state.container.hotel_service
        .list_hoteles(page, page_size)
        .await?;
    
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    let response = PaginatedResponse {
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

/// GET /api/v1/hoteles/cadena/:id_cadena - Listar hoteles de una cadena
#[instrument(skip(state, auth))]
pub async fn list_hoteles_by_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_cadena): Path<i32>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar hoteles".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    
    let (items, total) = state.container.hotel_service
        .list_hoteles_by_cadena(id_cadena, page, page_size)
        .await?;
    
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    let items: Vec<HotelResponse> = items.into_iter().map(HotelResponse::from).collect();
    let response = PaginatedResponse {
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

/// GET /api/v1/hoteles/:id - Obtener hotel por ID
#[instrument(skip(state, auth))]
pub async fn get_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente | UserRole::Hoteles) {
        return Err(ApplicationError::Forbidden("No tienes permisos para ver este hotel".to_string()));
    }
    
    let response = state.container.hotel_service
        .get_hotel(id)
        .await?;
    
    Ok(json_ok(response))
}

/// GET /api/v1/hoteles/mi-hotel - Obtener el hotel del usuario actual
#[instrument(skip(state, auth))]
pub async fn get_mi_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.hotel_service
        .get_mi_hotel(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    Ok(json_ok(response))
}
