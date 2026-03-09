//! GET handlers para File

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok};

use super::query_params::{DateRangeQuery, EntidadQuery};

/// Listar files con paginación
#[instrument(skip(state, _auth))]
pub async fn list_files(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Query(params): Query<PaginationParams>
) -> Result<impl IntoResponse, ApplicationError> {
    let (items, total, total_pages) = state.container.file_service
        .list_files(params.to_options())
        .await?;
    
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { 
            page: params.page, 
            page_size: params.page_size, 
            total, 
            total_pages,
        },
    }))
}

/// Obtener file por ID
#[instrument(skip(state, _auth))]
pub async fn get_file(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.file_service
        .get_file(id)
        .await?;
    
    Ok(json_ok(response))
}

/// Listar files por agencia
#[instrument(skip(state, _auth))]
pub async fn list_files_by_agencia(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(agencia_id): Path<i32>,
    Query(query): Query<EntidadQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_by_agencia(agencia_id, query.entidad.as_deref())
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files por rango de fechas
#[instrument(skip(state, _auth))]
pub async fn list_files_by_date_range(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Query(query): Query<DateRangeQuery>
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .search_files_by_date_range(query.from, query.to)
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files próximos (en los próximos 7 días)
#[instrument(skip(state, _auth))]
pub async fn list_files_upcoming(
    State(state): State<AppState>, 
    _auth: AuthUser
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_upcoming()
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files con pago pendiente
#[instrument(skip(state, _auth))]
pub async fn list_files_pending_payment(
    State(state): State<AppState>, 
    _auth: AuthUser
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_pending_payment()
        .await?;
    
    Ok(json_ok(files))
}
