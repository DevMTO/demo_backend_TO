//! Pago Handlers - Endpoints HTTP para gestión de pagos
//! 
//! Estos handlers solo orquestan HTTP:
//! - Reciben requests
//! - Validan formato
//! - Delegan lógica de negocio a PagoService
//! - Retornan respuestas JSON

use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreatePagoRequest, UpdatePagoRequest};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

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

/// Registrar nuevo pago
#[instrument(skip(state, auth, request))]
pub async fn create_pago(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreatePagoRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.pago_service
        .register_pago(
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_created(response))
}

/// Actualizar pago existente
#[instrument(skip(state, auth, request))]
pub async fn update_pago(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(request): Json<UpdatePagoRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.pago_service
        .update_pago(
            id, 
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}

/// Eliminar pago
#[instrument(skip(state, auth))]
pub async fn delete_pago(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.pago_service
        .delete_pago(
            id, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_deleted())
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

#[derive(serde::Serialize)]
pub struct FileBalanceResponse { 
    pub file_id: i32, 
    pub ingresos: BigDecimal, 
    pub egresos: BigDecimal, 
    pub balance: BigDecimal 
}

/// Obtener balance de un file
#[instrument(skip(state, _auth))]
pub async fn get_file_balance(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(file_id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    // Este endpoint usa directamente el repositorio para cálculos agregados
    let ingresos = state.container.pago_repository.sum_ingresos_by_file(file_id).await?;
    let egresos = state.container.pago_repository.sum_egresos_by_file(file_id).await?;
    let balance = state.container.pago_repository.get_balance_by_file(file_id).await?;
    
    Ok(json_ok(FileBalanceResponse { file_id, ingresos, egresos, balance }))
}
