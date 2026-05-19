//! GET handlers para Cadenas Hoteleras

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{
    PaginationParams, PaginatedResponse, PaginationInfo, json_ok,
};

/// GET /api/v1/cadenas-hoteleras - Listar todas las cadenas hoteleras
#[instrument(skip(state, auth))]
pub async fn list_cadenas(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar cadenas hoteleras".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    
    let (items, total) = state.container.cadena_hotelera_service
        .list_cadenas(page, page_size)
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

/// GET /api/v1/cadenas-hoteleras/:id - Obtener cadena hotelera por ID
#[instrument(skip(state, auth))]
pub async fn get_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena | UserRole::Hoteles) {
        return Err(ApplicationError::Forbidden("No tienes permisos para ver esta cadena hotelera".to_string()));
    }
    
    let response = state.container.cadena_hotelera_service
        .get_cadena(id)
        .await?;
    
    Ok(json_ok(response))
}

/// GET /api/v1/cadenas-hoteleras/mi-cadena - Obtener la cadena del usuario actual
#[instrument(skip(state, auth))]
pub async fn get_mi_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.cadena_hotelera_service
        .get_mi_cadena(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    Ok(json_ok(response))
}
