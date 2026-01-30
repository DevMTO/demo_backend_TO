//! GET handlers para Persona

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{
    PaginationParams, PaginatedResponse, PaginationInfo, json_ok,
};
use crate::application::dtos::PersonaResponse;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Listar personas con paginación
#[instrument(skip(state, _auth))]
pub async fn list_personas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("Listando personas - página: {}, tamaño: {}", params.page, params.page_size);
    
    let options = params.to_options();
    let (items, total, total_pages) = state.container.persona_service
        .list_personas(options)
        .await?;
    
    let response: PaginatedResponse<PersonaResponse> = PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page: params.page,
            page_size: params.page_size,
            total,
            total_pages,
        },
    };
    
    Ok(json_ok(response))
}

/// Obtener persona por ID
#[instrument(skip(state, _auth))]
pub async fn get_persona(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("Buscando persona ID: {}", id);
    
    let persona = state.container.persona_service
        .get_persona(id)
        .await?;
    
    Ok(json_ok(persona))
}

/// Buscar personas por texto
#[instrument(skip(state, _auth))]
pub async fn search_personas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("Buscando personas: {}", query.q);
    
    let response = state.container.persona_service
        .search_personas(&query.q)
        .await?;
    
    Ok(json_ok(response))
}
