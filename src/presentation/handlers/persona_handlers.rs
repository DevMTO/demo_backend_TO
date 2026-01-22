//! Persona Handlers - Endpoints HTTP para gestión de personas
//!
//! Los handlers solo manejan preocupaciones HTTP:
//! - Validación de requests
//! - Extracción de parámetros
//! - Conversión de respuestas
//!
//! La lógica de negocio está en PersonaService

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
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_deleted,
};

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

/// Crear nueva persona
#[instrument(skip(state, auth, request))]
pub async fn create_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreatePersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    info!("Creando persona: {} {}", request.nombre, request.apellidos);
    
    // Delegar al servicio
    let response = state.container.persona_service
        .create_persona(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona creada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_created(response))
}

/// Actualizar persona existente
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
    
    debug!("Actualizando persona ID: {}", id);
    
    // Delegar al servicio
    let response = state.container.persona_service
        .update_persona(
            id,
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona actualizada: {} (ID: {})", response.nombre, response.id);
    
    Ok(json_ok(response))
}

/// Eliminar persona
#[instrument(skip(state, auth))]
pub async fn delete_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("🗑️ Eliminando persona ID: {}", id);
    
    // Delegar al servicio
    state.container.persona_service
        .delete_persona(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona eliminada ID: {}", id);
    Ok(json_deleted())
}

/// Eliminación permanente de persona (hard delete) - Solo SuperAdmin
#[instrument(skip(state, auth))]
pub async fn hard_delete_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente personas".to_string()));
    }
    
    debug!("🗑️ HARD DELETE persona ID: {}", id);
    
    // Delegar al servicio
    state.container.persona_service
        .hard_delete_persona(
            id,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    info!("Persona ELIMINADA PERMANENTEMENTE ID: {}", id);
    Ok(json_deleted())
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Buscar personas por texto
#[instrument(skip(state, _auth))]
pub async fn search_personas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    debug!("Buscando personas: {}", query.q);
    
    // Delegar al servicio
    let response = state.container.persona_service
        .search_personas(&query.q)
        .await?;
    
    Ok(json_ok(response))
}
