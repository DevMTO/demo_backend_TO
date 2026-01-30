//! GET handlers para Conductor

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use tracing::instrument;

use crate::application::dtos::ConductorListResponse;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, json_ok};

/// GET /api/v1/conductores - Listar conductores con paginación
#[instrument(skip(state, auth))]
pub async fn list_conductores(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    
    // Si el usuario es Transportes, solo puede ver conductores de su transporte
    let (items, total) = if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        state.container.conductor_service.list_conductores_by_transporte_paginated(transporte_id, page_size, offset).await?
    } else {
        state.container.conductor_service.list_conductores(page_size, offset).await?
    };
    
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    
    Ok(json_ok(ConductorListResponse {
        items,
        total,
        page,
        page_size,
        total_pages,
    }))
}

/// GET /api/v1/conductores/:id - Obtener un conductor por ID
#[instrument(skip(state, auth))]
pub async fn get_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let conductor = state.container.conductor_service.get_conductor(id).await?;
    
    // Si el usuario es Transportes, verificar que el conductor pertenece a su transporte
    if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        if conductor.id_transporte != Some(transporte_id) {
            return Err(ApplicationError::Forbidden("No tiene acceso a este conductor".into()));
        }
    }
    
    Ok(json_ok(conductor))
}

/// GET /api/v1/transportes/:transporte_id/conductores - Listar conductores por transporte
#[instrument(skip(state, auth))]
pub async fn list_conductores_by_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(transporte_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Si el usuario es Transportes, verificar que solo accede a su propio transporte
    if auth.user.role == UserRole::Transportes {
        let user_transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        if user_transporte_id != transporte_id {
            return Err(ApplicationError::Forbidden("No tiene acceso a los conductores de este transporte".into()));
        }
    }
    
    let conductores = state.container.conductor_service.list_by_transporte(transporte_id).await?;
    Ok(json_ok(conductores))
}

/// GET /api/v1/conductores/available - Listar conductores disponibles
#[instrument(skip(state, auth))]
pub async fn list_conductores_available(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Si el usuario es Transportes, solo ver conductores disponibles de su transporte
    let conductores = if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        state.container.conductor_service.list_available_by_transporte(transporte_id).await?
    } else {
        state.container.conductor_service.list_available().await?
    };
    
    Ok(json_ok(conductores))
}
