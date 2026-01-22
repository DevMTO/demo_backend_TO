use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateVehiculoRequest, UpdateVehiculoRequest};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{json_ok, json_created, json_deleted, create_paginated_response};

/// Query params para listar vehículos
#[derive(Debug, Deserialize)]
pub struct VehiculosQueryParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filtrar por transporte específico
    pub id_transporte: Option<i32>,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

/// GET /api/v1/vehiculos - Listar vehículos con paginación y filtro opcional por transporte
#[instrument(skip(state, _auth))]
pub async fn list_vehiculos(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<VehiculosQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    
    // Si hay filtro por transporte, usar el método específico
    let (items, total) = if let Some(transporte_id) = params.id_transporte {
        let vehiculos = state.container.vehiculo_service.list_by_transporte(transporte_id).await?;
        let total = vehiculos.len() as i64;
        (vehiculos, total)
    } else {
        state.container.vehiculo_service.list_vehiculos(page_size, offset).await?
    };
    
    let response = create_paginated_response(items, total, page, page_size);
    Ok(json_ok(response))
}

/// GET /api/v1/vehiculos/:id - Obtener un vehículo por ID
#[instrument(skip(state, _auth))]
pub async fn get_vehiculo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculo = state.container.vehiculo_service.get_vehiculo(id).await?;
    Ok(json_ok(vehiculo))
}

/// POST /api/v1/vehiculos - Crear un nuevo vehículo
#[instrument(skip(state, auth, request))]
pub async fn create_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateVehiculoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let created = state.container.vehiculo_service
        .create_vehiculo(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Vehículo creado: {} (ID: {})", created.placa, created.id);
    Ok(json_created(created))
}

/// PUT /api/v1/vehiculos/:id - Actualizar un vehículo
#[instrument(skip(state, auth, request))]
pub async fn update_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateVehiculoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let updated = state.container.vehiculo_service
        .update_vehiculo(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Vehículo actualizado: {} (ID: {})", updated.placa, id);
    Ok(json_ok(updated))
}

/// DELETE /api/v1/vehiculos/:id - Eliminar un vehículo
#[instrument(skip(state, auth))]
pub async fn delete_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.vehiculo_service
        .delete_vehiculo(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("🗑️ Handler: Vehículo eliminado (ID: {})", id);
    Ok(json_deleted())
}

/// GET /api/v1/transportes/:transporte_id/vehiculos - Listar vehículos por transporte
#[instrument(skip(state, _auth))]
pub async fn list_vehiculos_by_transporte(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(transporte_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = state.container.vehiculo_service.list_by_transporte(transporte_id).await?;
    Ok(json_ok(vehiculos))
}

/// GET /api/v1/vehiculos/available - Listar vehículos disponibles
#[instrument(skip(state, _auth))]
pub async fn list_vehiculos_available(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = state.container.vehiculo_service.list_available().await?;
    Ok(json_ok(vehiculos))
}
