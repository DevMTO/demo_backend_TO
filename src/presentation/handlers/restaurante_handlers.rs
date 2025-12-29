use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateRestauranteRequest, UpdateRestauranteRequest, RestauranteResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[instrument(skip(state, _auth))]
pub async fn list_restaurantes(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let result = state.container.restaurante_repository.list_paginated(params.to_options()).await?;
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(RestauranteResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_restaurante(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    Ok(json_ok(RestauranteResponse::from(r)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_restaurante(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateRestauranteRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let created = state.container.restaurante_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Restaurante creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(RestauranteResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateRestauranteRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let result = state.container.restaurante_repository.update(&request.apply_to(r, Some(auth.user.id))).await?;
    Ok(json_ok(RestauranteResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let mut updated = r.clone();
    updated.is_active = false;
    updated.updated_by = Some(auth.user.id);
    updated.updated_at = chrono::Utc::now();
    state.container.restaurante_repository.update(&updated).await?;
    Ok(json_message("Restaurante desactivado correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_restaurante(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let r = state.container.restaurante_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", id)))?;
    let mut updated = r.clone();
    updated.is_active = true;
    updated.updated_by = Some(auth.user.id);
    updated.updated_at = chrono::Utc::now();
    state.container.restaurante_repository.update(&updated).await?;
    Ok(json_message("Restaurante restaurado correctamente"))
}

#[derive(Debug, serde::Deserialize)]
pub struct RestauranteSearchQuery { pub tipo_atencion: Option<String>, pub min_capacidad: Option<i32> }

#[instrument(skip(state, _auth))]
pub async fn search_restaurantes(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<RestauranteSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let result = if let Some(tipo) = query.tipo_atencion {
        state.container.restaurante_repository.find_by_tipo_atencion(&tipo).await?
    } else if let Some(min_cap) = query.min_capacidad {
        state.container.restaurante_repository.find_by_min_capacity(min_cap).await?
    } else {
        let paginated = state.container.restaurante_repository.list_paginated(crate::application::ports::PaginationOptions { limit: Some(100), offset: Some(0) }).await?;
        paginated.data
    };
    Ok(json_ok(result.into_iter().map(RestauranteResponse::from).collect::<Vec<_>>()))
}
