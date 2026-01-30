//! GET handlers para Pago

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use bigdecimal::BigDecimal;
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok};

#[derive(serde::Serialize)]
pub struct FileBalanceResponse { 
    pub file_id: i32, 
    pub ingresos: BigDecimal, 
    pub egresos: BigDecimal, 
    pub balance: BigDecimal 
}

/// Listar pagos con paginación
#[instrument(skip(state, _auth))]
pub async fn list_pagos(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Query(params): Query<PaginationParams>
) -> Result<impl IntoResponse, ApplicationError> {
    let (items, total, total_pages) = state.container.pago_service
        .list_pagos(params.to_options())
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

/// Obtener pago por ID
#[instrument(skip(state, _auth))]
pub async fn get_pago(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.pago_service
        .get_pago(id)
        .await?;
    
    Ok(json_ok(response))
}

/// Listar pagos por file
#[instrument(skip(state, _auth))]
pub async fn list_pagos_by_file(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(file_id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let pagos = state.container.pago_service
        .list_pagos_by_file(file_id)
        .await?;
    
    Ok(json_ok(pagos))
}

/// Obtener balance de un file
#[instrument(skip(state, _auth))]
pub async fn get_file_balance(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(file_id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let ingresos = state.container.pago_repository.sum_ingresos_by_file(file_id).await?;
    let egresos = state.container.pago_repository.sum_egresos_by_file(file_id).await?;
    let balance = state.container.pago_repository.get_balance_by_file(file_id).await?;
    
    Ok(json_ok(FileBalanceResponse { file_id, ingresos, egresos, balance }))
}
