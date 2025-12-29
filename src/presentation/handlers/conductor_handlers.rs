use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateConductorRequest, UpdateConductorRequest, ConductorResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_conductores(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.conductor_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(ConductorResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_conductor(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let c = state.container.conductor_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
    Ok(json_ok(ConductorResponse::from(c)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_conductor(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateConductorRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.conductor_repository.exists_by_brevete(&request.nro_brevete).await? {
        return Err(ApplicationError::Conflict(format!("Brevete {} ya existe", request.nro_brevete)));
    }
    let created = state.container.conductor_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Conductor creado: {} (ID: {})", created.nro_brevete, created.id);
    Ok(json_created(ConductorResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_conductor(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateConductorRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let c = state.container.conductor_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id)))?;
    let result = state.container.conductor_repository.update(&request.apply_to(c, Some(auth.user.id))).await?;
    Ok(json_ok(ConductorResponse::from(result)))
}

#[instrument(skip(state, _auth))]
pub async fn delete_conductor(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.conductor_repository.delete(id).await? { return Err(ApplicationError::NotFound(format!("Conductor {} no encontrado", id))); }
    Ok(json_deleted())
}

#[instrument(skip(state, _auth))]
pub async fn list_conductores_by_transporte(State(state): State<AppState>, _auth: AuthUser, Path(transporte_id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_repository.find_by_transporte(transporte_id).await?;
    Ok(json_ok(conductores.into_iter().map(ConductorResponse::from).collect::<Vec<_>>()))
}

#[instrument(skip(state, _auth))]
pub async fn list_conductores_available(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_repository.list_available().await?;
    Ok(json_ok(conductores.into_iter().map(ConductorResponse::from).collect::<Vec<_>>()))
}
