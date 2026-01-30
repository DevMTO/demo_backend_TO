//! GET handlers para Restaurante

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use serde::Deserialize;
use tracing::instrument;

use crate::application::ports::PaginationOptions;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok};

#[derive(Debug, Deserialize)]
pub struct RestauranteSearchQuery {
    pub tipo_atencion: Option<String>,
    pub min_capacidad: Option<i32>,
}

/// GET /api/v1/restaurantes - Listar restaurantes con paginación
#[instrument(skip(state, _auth))]
pub async fn list_restaurantes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let limit = params.page_size.min(100);
    let offset = (params.page - 1).max(0) * limit;
    
    let (items, total) = state.container.restaurante_service.list_restaurantes(limit, offset).await?;
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

/// GET /api/v1/restaurantes/:id - Obtener un restaurante por ID
#[instrument(skip(state, _auth))]
pub async fn get_restaurante(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let restaurante = state.container.restaurante_service.get_restaurante(id).await?;
    Ok(json_ok(restaurante))
}

/// GET /api/v1/restaurantes/search - Buscar restaurantes por tipo de atención o capacidad
#[instrument(skip(state, _auth))]
pub async fn search_restaurantes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<RestauranteSearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let result = if let Some(tipo) = query.tipo_atencion {
        state.container.restaurante_service.search_by_tipo_atencion(&tipo).await?
    } else if let Some(min_cap) = query.min_capacidad {
        state.container.restaurante_service.search_by_min_capacity(min_cap).await?
    } else {
        state.container.restaurante_service.list_simple(PaginationOptions {
            limit: Some(100),
            offset: Some(0),
        }).await?
    };
    
    Ok(json_ok(result))
}
