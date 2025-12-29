use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use chrono::NaiveDate;
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateFileRequest, UpdateFileRequest, FileResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_files(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.file_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(FileResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_file(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let f = state.container.file_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
    Ok(json_ok(FileResponse::from(f)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_file(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateFileRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para crear el file
    let response = state.container.create_file_use_case
        .execute(request, auth.user.id)
        .await?;
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_file(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateFileRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_file_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn delete_file(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.file_repository.delete(id).await? { return Err(ApplicationError::NotFound(format!("File {} no encontrado", id))); }
    Ok(json_deleted())
}

#[instrument(skip(state, _auth))]
pub async fn list_files_by_agencia(State(state): State<AppState>, _auth: AuthUser, Path(agencia_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_repository.find_by_agencia(agencia_id).await?;
    Ok(json_ok(files.into_iter().map(FileResponse::from).collect::<Vec<_>>()))
}

#[derive(Debug, serde::Deserialize)]
pub struct DateRangeQuery { pub from: NaiveDate, pub to: NaiveDate }

#[instrument(skip(state, _auth))]
pub async fn list_files_by_date_range(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<DateRangeQuery>) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para búsqueda por rango
    let response = state.container.search_files_use_case
        .by_date_range(query.from, query.to)
        .await?;
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn list_files_upcoming(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para files próximos
    let response = state.container.search_files_use_case.upcoming().await?;
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn list_files_pending_payment(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para files con pago pendiente
    let response = state.container.search_files_use_case.pending_payment().await?;
    Ok(json_ok(response))
}
