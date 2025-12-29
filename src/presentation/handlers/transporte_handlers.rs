use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateTransporteRequest, UpdateTransporteRequest, TransporteResponse};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[instrument(skip(state, _auth))]
pub async fn list_transportes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.transporte_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(TransporteResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_transporte(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let t = state.container.transporte_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
    Ok(json_ok(TransporteResponse::from(t)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.transporte_repository.exists_by_ruc(&request.ruc).await? {
        return Err(ApplicationError::Conflict(format!("RUC {} ya existe", request.ruc)));
    }
    let created = state.container.transporte_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Transporte creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(TransporteResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let t = state.container.transporte_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
    let result = state.container.transporte_repository.update(&request.apply_to(t, Some(auth.user.id))).await?;
    Ok(json_ok(TransporteResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.transporte_repository.soft_delete(id, auth.user.id).await? {
        return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
    }
    Ok(json_message("Transporte desactivado"))
}

#[instrument(skip(state, auth))]
pub async fn restore_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.transporte_repository.restore(id, auth.user.id).await? {
        return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
    }
    Ok(json_message("Transporte restaurado"))
}

#[instrument(skip(state, _auth))]
pub async fn list_transportes_with_vehicles(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let transportes = state.container.transporte_repository.find_with_available_vehicles().await?;
    let response: Vec<TransporteResponse> = transportes.into_iter().map(Into::into).collect();
    Ok(json_ok(response))
}
