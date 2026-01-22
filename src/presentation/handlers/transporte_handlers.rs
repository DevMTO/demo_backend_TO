use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{
    CreateTransporteRequest, UpdateTransporteRequest, UpdateTransporteInterfazRequest, 
    TransporteResponse
};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

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
/// 
/// Busca primero por id_entidad si el usuario es de un transporte,
/// o por encargado si el usuario es el responsable de un transporte.
#[instrument(skip(state, auth))]
pub async fn get_mi_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚐 Buscando transporte para usuario '{}' (id_persona: {:?}, id_entidad: {:?}, role: {:?})", 
        auth.user.username, auth.user.id_persona, auth.user.id_entidad, auth.user.role);
    
    let mut transporte: Option<TransporteResponse> = None;
    
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    // Primero intentar por id_entidad si el usuario está relacionado con un transporte
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            transporte = Some(state.container.transporte_service.get_transporte(id_entidad).await?);
            if transporte.is_some() {
                info!("Transporte encontrado por id_entidad: {}", id_entidad);
            }
        }
    }
    
    // Si no se encontró, buscar por encargado (id_persona)
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

/// PUT /api/v1/transportes/me - Actualizar mi propio transporte
/// 
/// Permite a usuarios de tipo Transporte o encargados actualizar su transporte.
/// Solo pueden actualizar: paleta_colores, direccion, telefono, correo, media.
/// No pueden modificar: ruc, nombre, encargado, estado.
#[instrument(skip(state, auth, request))]
pub async fn update_mi_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚐 Usuario '{}' intenta actualizar su transporte", auth.user.username);
    
    // Buscar el transporte del usuario
    let transporte_id = find_user_transporte_id(&state, &auth).await?;
    
    // Delegar al servicio (que ya limita los campos)
    let result = state.container.transporte_service
        .update_my_transporte(transporte_id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Transporte actualizado por su usuario: {}", result.nombre);
    Ok(json_ok(result))
}

/// PATCH /api/v1/transportes/me/interfaz - Actualizar solo la interfaz de mi transporte
/// 
/// Endpoint PATCH que permite actualizar solo logo y paleta_colores.
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_transporte_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateTransporteInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🎨 Usuario '{}' actualiza interfaz de su transporte", auth.user.username);
    
    // Buscar el transporte del usuario
    let transporte_id = find_user_transporte_id(&state, &auth).await?;
    
    // Delegar al servicio
    let result = state.container.transporte_service
        .update_interface(transporte_id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Interfaz de transporte '{}' actualizada", result.nombre);
    Ok(json_ok(result))
}

/// POST /api/v1/transportes - Crear un nuevo transporte
#[instrument(skip(state, auth, request))]
pub async fn create_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let created = state.container.transporte_service
        .create_transporte(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Handler: Transporte creado: {} (ID: {})", created.nombre, created.id);
    Ok(json_created(created))
}

/// PUT /api/v1/transportes/:id - Actualizar un transporte (por admin)
#[instrument(skip(state, auth, request))]
pub async fn update_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Validar request
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar al servicio
    let updated = state.container.transporte_service
        .update_transporte(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("✏️ Handler: Transporte actualizado: {} (ID: {})", updated.nombre, id);
    Ok(json_ok(updated))
}

/// DELETE /api/v1/transportes/:id - Desactivar un transporte (soft delete)
#[instrument(skip(state, auth))]
pub async fn delete_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.transporte_service
        .deactivate_transporte(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("🗑️ Handler: Transporte desactivado (ID: {})", id);
    Ok(json_message("Transporte desactivado"))
}

/// POST /api/v1/transportes/:id/restore - Restaurar un transporte desactivado
#[instrument(skip(state, auth))]
pub async fn restore_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar al servicio
    state.container.transporte_service
        .restore_transporte(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("♻️ Handler: Transporte restaurado (ID: {})", id);
    Ok(json_message("Transporte restaurado"))
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

// ===== Función auxiliar =====

/// Busca el ID del transporte asociado al usuario
async fn find_user_transporte_id(state: &AppState, auth: &AuthUser) -> Result<i32, ApplicationError> {
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    // Primero intentar por id_entidad
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            return Ok(id_entidad);
        }
    }
    
    // Si no, buscar por encargado (id_persona)
    if let Some(persona_id) = auth.user.id_persona {
        if let Some(transporte) = state.container.transporte_service.find_by_encargado(persona_id).await? {
            return Ok(transporte.id);
        }
    }
    
    Err(ApplicationError::NotFound("No tienes un transporte asociado".to_string()))
}
