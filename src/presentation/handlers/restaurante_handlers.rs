use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateRestauranteRequest, UpdateRestauranteRequest};
use crate::application::ports::PaginationOptions;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

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

/// POST /api/v1/restaurantes - Crear un nuevo restaurante
#[instrument(skip(state, auth, request))]
pub async fn create_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateRestauranteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let created = state.container.restaurante_service
        .create_restaurante(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✅ Handler: Restaurante creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(created))
}

/// PUT /api/v1/restaurantes/:id - Actualizar un restaurante
#[instrument(skip(state, auth, request))]
pub async fn update_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateRestauranteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let updated = state.container.restaurante_service
        .update_restaurante(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Restaurante actualizado: {} (ID: {})", updated.nombre, id);
    Ok(json_ok(updated))
}

/// DELETE /api/v1/restaurantes/:id - Desactivar un restaurante (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.restaurante_service
        .deactivate_restaurante(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("🗑️ Handler: Restaurante desactivado (ID: {})", id);
    Ok(json_message("Restaurante desactivado correctamente"))
}

/// POST /api/v1/restaurantes/:id/restore - Restaurar un restaurante desactivado
#[instrument(skip(state, auth))]
pub async fn restore_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.restaurante_service
        .restore_restaurante(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("♻️ Handler: Restaurante restaurado (ID: {})", id);
    Ok(json_message("Restaurante restaurado correctamente"))
}

/// Query params para búsqueda de restaurantes
#[derive(Debug, serde::Deserialize)]
pub struct RestauranteSearchQuery {
    pub tipo_atencion: Option<String>,
    pub min_capacidad: Option<i32>,
}

/// GET /api/v1/restaurantes/search - Buscar restaurantes por tipo de atención o capacidad
#[instrument(skip(state, _auth))]
pub async fn search_restaurantes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<RestauranteSearchQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let result = if let Some(tipo) = query.tipo_atencion {
        // Buscar por tipo de atención
        state.container.restaurante_service.search_by_tipo_atencion(&tipo).await?
    } else if let Some(min_cap) = query.min_capacidad {
        // Buscar por capacidad mínima
        state.container.restaurante_service.search_by_min_capacity(min_cap).await?
    } else {
        // Listar todos sin filtro específico
        state.container.restaurante_service.list_simple(PaginationOptions {
            limit: Some(100),
            offset: Some(0),
        }).await?
    };
    
    Ok(json_ok(result))
}
