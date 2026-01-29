use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateConductorRequest, UpdateConductorRequest, ConductorListResponse};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, json_ok, json_created, json_deleted};

/// GET /api/v1/conductores - Listar conductores con paginación
#[instrument(skip(state, _auth))]
pub async fn list_conductores(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    
    let (items, total) = state.container.conductor_service.list_conductores(page_size, offset).await?;
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
#[instrument(skip(state, _auth))]
pub async fn get_conductor(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let conductor = state.container.conductor_service.get_conductor(id).await?;
    Ok(json_ok(conductor))
}

/// POST /api/v1/conductores - Crear un nuevo conductor
#[instrument(skip(state, auth, request))]
pub async fn create_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateConductorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let created = state.container.conductor_service
        .create_conductor(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Conductor creado: {} (ID: {})", created.nro_brevete, created.id);
    Ok(json_created(created))
}

/// PUT /api/v1/conductores/:id - Actualizar un conductor
#[instrument(skip(state, auth, request))]
pub async fn update_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateConductorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let updated = state.container.conductor_service
        .update_conductor(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Conductor actualizado: {} (ID: {})", updated.nro_brevete, id);
    Ok(json_ok(updated))
}

/// DELETE /api/v1/conductores/:id - Eliminar un conductor
#[instrument(skip(state, auth))]
pub async fn delete_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.conductor_service
        .delete_conductor(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Conductor eliminado (ID: {})", id);
    Ok(json_deleted())
}

/// DELETE /api/v1/conductores/:id/hard-delete - Eliminación permanente (Solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn hard_delete_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario autenticado es SuperAdmin
    if auth.user.role != UserRole::SuperAdmin {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente conductores".to_string()));
    }
    
    // Delegar al servicio
    state.container.conductor_service
        .hard_delete_conductor(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("[DELETE] Handler: Conductor ELIMINADO PERMANENTEMENTE (ID: {})", id);
    Ok(json_deleted())
}

/// GET /api/v1/transportes/:transporte_id/conductores - Listar conductores por transporte
#[instrument(skip(state, _auth))]
pub async fn list_conductores_by_transporte(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(transporte_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_service.list_by_transporte(transporte_id).await?;
    Ok(json_ok(conductores))
}

/// GET /api/v1/conductores/available - Listar conductores disponibles
#[instrument(skip(state, _auth))]
pub async fn list_conductores_available(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let conductores = state.container.conductor_service.list_available().await?;
    Ok(json_ok(conductores))
}
