use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{
    CreateAgenciaRequest, UpdateAgenciaRequest, UpdateAgenciaInterfazRequest, 
    AgenciaResponse,
};
use crate::domain::entities::{Agencia, EntityType, UserRole};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{
    PaginationParams, PaginatedResponse, PaginationInfo,
    json_ok, json_created, json_message,
};

#[instrument(skip(state, auth))]
pub async fn list_agencias(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden listar todas las agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para listar agencias".to_string()));
    }
    
    let page = params.page;
    let page_size = params.page_size;
    
    // Delegar TODA la lógica al servicio
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

#[instrument(skip(state, auth))]
pub async fn get_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden obtener cualquier agencia por ID
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para ver esta agencia".to_string()));
    }
    
    // Delegar TODA la lógica al servicio
    let response = state.container.agencia_service
        .get_agencia(id)
        .await?;
    
    Ok(json_ok(response))
}

/// Obtener la agencia del usuario actual
#[instrument(skip(state, auth))]
pub async fn get_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Delegar TODA la lógica al servicio
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

/// Actualizar mi propia agencia (campos limitados)
#[instrument(skip(state, auth, request))]
pub async fn update_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Primero obtener mi agencia para saber su ID
    let mi_agencia = state.container.agencia_service
        .get_mi_agencia(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    // Crear un request limitado: solo permitir ciertos campos
    let limited_request = UpdateAgenciaRequest {
        nombre: None,
        ruc: None,
        direccion: request.direccion,
        telefono: request.telefono,
        correo: request.correo,
        encargado: None,
        paleta_colores: request.paleta_colores,
        media: request.media,
        is_active: None,
    };
    
    // Delegar al servicio
    let response = state.container.agencia_service
        .update_agencia(
            mi_agencia.id,
            limited_request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}

/// Actualizar solo la interfaz de mi agencia (logo y paleta de colores)
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_agencia_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Primero obtener mi agencia
    let mi_agencia = state.container.agencia_service
        .get_mi_agencia(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    // Para la interfaz, usamos el repositorio directamente ya que es una operación muy específica
    let old_agencia = state.container.agencia_repository
        .find_by_id(mi_agencia.id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Agencia no encontrada".to_string()))?;
    
    // Aplicar cambios solo de interfaz
    let updated = request.apply_to(old_agencia.clone(), Some(auth.user.id));
    let result = state.container.agencia_repository.update(&updated).await?;
    
    // Logging
    let _ = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        mi_agencia.id,
        Some(&old_agencia),
        Some(&result),
        Some(vec!["paleta_colores".to_string(), "media".to_string()]),
        None,
    ).await;
    
    Ok(json_ok(AgenciaResponse::from(result)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden crear agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para crear agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let response = state.container.agencia_service
        .create_agencia(request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_created(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden actualizar agencias por ID
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para actualizar agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let response = state.container.agencia_service
        .update_agencia(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth))]
pub async fn delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden desactivar agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para desactivar agencias".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    state.container.agencia_service
        .delete_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia desactivada correctamente"))
}

#[instrument(skip(state, auth))]
pub async fn restore_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden restaurar agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar agencias".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones)
    let _response = state.container.agencia_service
        .restore_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia restaurada correctamente"))
}

/// Eliminación permanente de agencia (SOLO SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn hard_delete_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin puede eliminar permanentemente agencias
    if !matches!(auth.user.role, UserRole::SuperAdmin) {
        return Err(ApplicationError::Forbidden("Solo SuperAdmin puede eliminar permanentemente agencias".to_string()));
    }
    
    // Delegar TODA la lógica al servicio (validaciones, logging, notificaciones críticas)
    state.container.agencia_service
        .hard_delete_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia eliminada permanentemente"))
}

#[instrument(skip(state, auth))]
pub async fn get_agencia_by_ruc(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ruc): Path<String>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden buscar agencias por RUC
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para buscar agencias".to_string()));
    }
    
    // Delegar TODA la lógica al servicio
    let response = state.container.agencia_service
        .get_agencia_by_ruc(&ruc)
        .await?;
    
    Ok(json_ok(response))
}
