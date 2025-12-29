use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{
    CreateAgenciaRequest, UpdateAgenciaRequest, AgenciaResponse,
};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_message,
};

#[instrument(skip(state, _auth))]
pub async fn list_agencias(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let options = params.to_options();
    let result = state.container.agencia_repository
        .list_paginated(options)
        .await?;
    
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    let response: PaginatedResponse<AgenciaResponse> = PaginatedResponse {
        items: result.data.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo {
            page,
            page_size,
            total: result.total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn get_agencia(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let agencia = state.container.agencia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}

#[instrument(skip(state, auth, request))]
pub async fn create_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para crear la agencia
    let response = state.container.create_agencia_use_case
        .execute(request, auth.user.id)
        .await?;
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_agencia_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth))]
pub async fn delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para desactivar
    state.container.deactivate_agencia_use_case
        .execute(id, auth.user.id)
        .await?;
    
    Ok(json_message("Agencia desactivada correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Usar el caso de uso para restaurar
    state.container.restore_agencia_use_case
        .execute(id, auth.user.id)
        .await?;
    
    Ok(json_message("Agencia restaurada correctamente"))
}

#[instrument(skip(state, _auth))]
pub async fn get_agencia_by_ruc(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(ruc): Path<String>,
) -> Result<impl IntoResponse, ApplicationError> {
    let agencia = state.container.agencia_repository
        .find_by_ruc(&ruc)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Agencia con RUC {} no encontrada", ruc)))?;
    
    let response: AgenciaResponse = agencia.into();
    Ok(json_ok(response))
}
