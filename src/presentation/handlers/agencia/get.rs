//! GET handlers para Agencias

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{
    PaginationParams, PaginatedResponse, PaginationInfo, json_ok,
};

/// GET /api/v1/agencias - Listar todas las agencias
#[instrument(skip(state, auth))]
pub async fn list_agencias(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar agencias".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    
    let (items, total) = state.container.agencia_service
        .list_agencias(page, page_size)
        .await?;
    
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    let response = PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page,
            page_size,
            total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

/// GET /api/v1/agencias/:id - Obtener agencia por ID
#[instrument(skip(state, auth))]
pub async fn get_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para ver esta agencia".to_string()));
    }
    
    let response = state.container.agencia_service
        .get_agencia(id)
        .await?;
    
    Ok(json_ok(response))
}

/// GET /api/v1/agencias/mi-agencia - Obtener la agencia del usuario actual
#[instrument(skip(state, auth))]
pub async fn get_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.agencia_service
        .get_mi_agencia(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    Ok(json_ok(response))
}

/// GET /api/v1/agencias/ruc/:ruc - Obtener agencia por RUC
#[instrument(skip(state, auth))]
pub async fn get_agencia_by_ruc(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ruc): Path<String>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para buscar agencias".to_string()));
    }
    
    let response = state.container.agencia_service
        .get_agencia_by_ruc(&ruc)
        .await?;
    
    Ok(json_ok(response))
}
