//! GET handlers para Vehiculo

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, create_paginated_response};
use crate::presentation::extractors::AuthUser;
use super::query_params::VehiculosQueryParams;

/// Listar vehículos con paginación
/// - Para SuperAdmin/Admin: lista todos los vehículos
/// - Para Transportes: lista solo los vehículos de su transporte (id_entidad)
#[instrument(skip(state, auth))]
pub async fn list_vehiculos(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<VehiculosQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page.unwrap_or(1) as i64;
    let page_size = params.page_size.unwrap_or(10).min(100).max(1) as i64;
    let offset = (page - 1) * page_size;
    
    // Si es transportes, filtrar solo por su id_entidad
    let (vehiculos, total) = if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        state.container.vehiculo_service
            .list_vehiculos_by_transporte_paginated(transporte_id, page_size, offset)
            .await?
    } else {
        state.container.vehiculo_service
            .list_vehiculos(page_size, offset)
            .await?
    };
    
    let response = create_paginated_response(vehiculos, total, page, page_size);
    
    Ok(json_ok(response))
}

/// Obtener un vehículo por ID
/// - Para Transportes: solo puede ver vehículos de su transporte
#[instrument(skip(state, auth))]
pub async fn get_vehiculo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculo = state.container.vehiculo_service
        .get_vehiculo(id)
        .await?;
    
    // Si es transportes, verificar que el vehículo pertenece a su transporte
    if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        if vehiculo.id_transporte != transporte_id {
            return Err(ApplicationError::Forbidden("No tiene acceso a este vehículo".into()));
        }
    }
    
    Ok(json_ok(vehiculo))
}

/// Listar vehículos por transporte
#[instrument(skip(state, auth))]
pub async fn list_vehiculos_by_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(transporte_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Si es transportes, solo puede ver los de su transporte
    if auth.user.role == UserRole::Transportes {
        let user_transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        if user_transporte_id != transporte_id {
            return Err(ApplicationError::Forbidden("No tiene acceso a los vehículos de este transporte".into()));
        }
    }
    
    let vehiculos = state.container.vehiculo_service
        .list_by_transporte(transporte_id)
        .await?;
    
    Ok(json_ok(vehiculos))
}

/// Listar vehículos disponibles
/// - Para Transportes: lista solo los vehículos disponibles de su transporte
#[instrument(skip(state, auth))]
pub async fn list_vehiculos_available(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = if auth.user.role == UserRole::Transportes {
        let transporte_id = auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario transportes sin id_entidad asignado".into())
        })?;
        state.container.vehiculo_service
            .list_available_by_transporte(transporte_id)
            .await?
    } else {
        state.container.vehiculo_service
            .list_available()
            .await?
    };
    
    Ok(json_ok(vehiculos))
}
