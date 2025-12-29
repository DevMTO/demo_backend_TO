use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, info, instrument};
use validator::Validate;

use crate::application::dtos::{
    CreatePersonaRequest, UpdatePersonaRequest, PersonaResponse,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_deleted,
};

#[instrument(skip(state, _auth))]
pub async fn list_personas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("📋 Listando personas - página: {}, tamaño: {}", params.page, params.page_size);
    
    let options = params.to_options();
    let result = state.container.persona_repository
        .list_paginated(options.clone())
        .await?;
    
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    let response: PaginatedResponse<PersonaResponse> = PaginatedResponse {
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
pub async fn get_persona(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("🔍 Buscando persona ID: {}", id);
    
    let persona = state.container.persona_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)))?;
    
    let response: PersonaResponse = persona.into();
    Ok(json_ok(response))
}

#[instrument(skip(state, auth, request))]
pub async fn create_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreatePersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    info!("📝 Creando persona: {} {}", request.nombre, request.apellidos);
    
    // Usar el caso de uso para crear la persona
    let response = state.container.create_persona_use_case
        .execute(request, auth.user.id)
        .await?;
    
    info!("✅ Persona creada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdatePersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    debug!("📝 Actualizando persona ID: {}", id);
    
    // Usar el caso de uso para actualizar
    let response = state.container.update_persona_use_case
        .execute(id, request, auth.user.id)
        .await?;
    
    info!("✅ Persona actualizada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_ok(response))
}

#[instrument(skip(state, _auth))]
pub async fn delete_persona(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("🗑️ Eliminando persona ID: {}", id);
    
    let deleted = state.container.persona_repository.delete(id).await?;
    
    if !deleted {
        return Err(ApplicationError::NotFound(format!("Persona con ID {} no encontrada", id)));
    }
    
    info!("✅ Persona eliminada ID: {}", id);
    Ok(json_deleted())
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[instrument(skip(state, _auth))]
pub async fn search_personas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("🔍 Buscando personas: {}", query.q);
    
    // Usar el caso de uso para buscar
    let response = state.container.search_personas_use_case
        .execute(&query.q)
        .await?;
    
    Ok(json_ok(response))
}
