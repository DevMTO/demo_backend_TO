//! File Handlers - Endpoints HTTP para gestión de files (paquetes turísticos)
//! 
//! Estos handlers solo orquestan HTTP:
//! - Reciben requests
//! - Validan formato
//! - Delegan lógica de negocio a FileService
//! - Retornan respuestas JSON

use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use chrono::NaiveDate;
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateFileRequest, UpdateFileRequest};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

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

/// Crear nuevo file
#[instrument(skip(state, auth, request))]
pub async fn create_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.file_service
        .create_file(
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_created(response))
}

/// Actualizar file existente
#[instrument(skip(state, auth, request))]
pub async fn update_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(request): Json<UpdateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.file_service
        .update_file(
            id, 
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}

/// Eliminar file
#[instrument(skip(state, auth))]
pub async fn delete_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_service
        .delete_file(
            id, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_deleted())
}

/// Listar files por agencia
#[instrument(skip(state, _auth))]
pub async fn list_files_by_agencia(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(agencia_id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_by_agencia(agencia_id)
        .await?;
    
    Ok(json_ok(files))
}

#[derive(Debug, serde::Deserialize)]
pub struct DateRangeQuery { 
    pub from: NaiveDate, 
    pub to: NaiveDate 
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
