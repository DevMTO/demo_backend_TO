//! GET handlers para Transporte

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::{info, instrument};

use crate::application::dtos::TransporteResponse;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok};

/// GET /api/v1/transportes - Listar transportes con paginación
#[instrument(skip(state, _auth))]
pub async fn list_transportes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let limit = params.page_size.min(100);
    let offset = (params.page - 1).max(0) * limit;
    
    let (items, total) = state.container.transporte_service.list_transportes(limit, offset).await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;
    
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page: params.page,
            page_size: limit,
            total,
            total_pages,
        },
    }))
}

/// GET /api/v1/transportes/:id - Obtener un transporte por ID
#[instrument(skip(state, _auth))]
pub async fn get_transporte(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let transporte = state.container.transporte_service.get_transporte(id).await?;
    Ok(json_ok(transporte))
}

/// GET /api/v1/transportes/me - Obtener el transporte del usuario autenticado
#[instrument(skip(state, auth))]
pub async fn get_mi_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚐 Buscando transporte para usuario '{}' (id_persona: {:?}, id_entidad: {:?}, role: {:?})", 
        auth.user.username, auth.user.id_persona, auth.user.id_entidad, auth.user.role);
    
    let mut transporte: Option<TransporteResponse> = None;
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            transporte = Some(state.container.transporte_service.get_transporte(id_entidad).await?);
            if transporte.is_some() {
                info!("Transporte encontrado por id_entidad: {}", id_entidad);
            }
        }
    }
    
    if transporte.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            transporte = state.container.transporte_service.find_by_encargado(persona_id).await?;
            if transporte.is_some() {
                info!("Transporte encontrado por encargado (persona_id: {})", persona_id);
            }
        }
    }
    
    match transporte {
        Some(t) => Ok(json_ok(t)),
        None => {
            info!("ℹ️ Usuario '{}' no tiene transporte asociado", auth.user.username);
            Err(ApplicationError::NotFound("No tienes un transporte asociado".to_string()))
        }
    }
}

/// GET /api/v1/transportes/with-vehicles - Listar transportes con vehículos disponibles
#[instrument(skip(state, _auth))]
pub async fn list_transportes_with_vehicles(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let transportes = state.container.transporte_service.list_with_available_vehicles().await?;
    Ok(json_ok(transportes))
}
