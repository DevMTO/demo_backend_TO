use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreatePagoRequest, UpdatePagoRequest, PagoResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_pagos(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.pago_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(PagoResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_pago(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let p = state.container.pago_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))?;
    Ok(json_ok(PagoResponse::from(p)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_pago(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreatePagoRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para registrar el pago
    let response = state.container.register_pago_use_case
        .execute(request, auth.user.id)
        .await?;
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_pago(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdatePagoRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_pago_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn delete_pago(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.pago_repository.delete(id).await? { return Err(ApplicationError::NotFound(format!("Pago {} no encontrado", id))); }
    Ok(json_deleted())
}

#[instrument(skip(state, _auth))]
pub async fn list_pagos_by_file(State(state): State<AppState>, _auth: AuthUser, Path(file_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let pagos = state.container.pago_repository.find_by_file(file_id).await?;
    Ok(json_ok(pagos.into_iter().map(PagoResponse::from).collect::<Vec<_>>()))
}

#[derive(serde::Serialize)]
pub struct FileBalanceResponse { pub file_id: i32, pub ingresos: BigDecimal, pub egresos: BigDecimal, pub balance: BigDecimal }

#[instrument(skip(state, _auth))]
pub async fn get_file_balance(State(state): State<AppState>, _auth: AuthUser, Path(file_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let ingresos = state.container.pago_repository.sum_ingresos_by_file(file_id).await?;
    let egresos = state.container.pago_repository.sum_egresos_by_file(file_id).await?;
    let balance = state.container.pago_repository.get_balance_by_file(file_id).await?;
    Ok(json_ok(FileBalanceResponse { file_id, ingresos, egresos, balance }))
}
