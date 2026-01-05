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
use crate::domain::entities::{EntityType, UserRole, NotificationType, NotificationCategory, NotificationPriority};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[instrument(skip(state, _auth))]
pub async fn list_transportes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let limit = params.page_size.min(100);
    let offset = (params.page - 1).max(0) * limit;
    let (items, total) = state.container.transporte_repository.list_with_encargado(limit, offset).await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { page: params.page, page_size: limit, total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_transporte(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let t = state.container.transporte_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
    Ok(json_ok(TransporteResponse::from(t)))
}

/// Obtiene el transporte asociado al usuario autenticado.
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
    
    let mut transporte: Option<crate::domain::entities::Transporte> = None;
    
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    // Primero intentar por id_entidad si el usuario está relacionado con un transporte
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            transporte = state.container.transporte_repository
                .find_by_id(id_entidad)
                .await?;
            if transporte.is_some() {
                info!("✅ Transporte encontrado por id_entidad: {}", id_entidad);
            }
        }
    }
    
    // Si no se encontró, buscar por encargado (id_persona)
    if transporte.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            transporte = state.container.transporte_repository
                .find_by_encargado(persona_id)
                .await?;
            if transporte.is_some() {
                info!("✅ Transporte encontrado por encargado (persona_id: {})", persona_id);
            }
        }
    }
    
    match transporte {
        Some(t) => {
            let response: TransporteResponse = t.into();
            Ok(json_ok(response))
        }
        None => {
            info!("ℹ️ Usuario '{}' no tiene transporte asociado", auth.user.username);
            Err(ApplicationError::NotFound("No tienes un transporte asociado".to_string()))
        }
    }
}

/// Actualizar mi propio transporte
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
    let mut transporte: Option<crate::domain::entities::Transporte> = None;
    
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    // Intentar por id_entidad si el usuario está relacionado con un transporte
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            transporte = state.container.transporte_repository
                .find_by_id(id_entidad)
                .await?;
        }
    }
    
    // Si no se encontró, buscar por encargado (id_persona)
    if transporte.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            transporte = state.container.transporte_repository
                .find_by_encargado(persona_id)
                .await?;
        }
    }
    
    let old_transporte = transporte.ok_or_else(|| {
        ApplicationError::NotFound("No tienes un transporte asociado".to_string())
    })?;
    
    let transporte_id = old_transporte.id;
    
    // Crear un request limitado: solo permitir ciertos campos
    let limited_request = UpdateTransporteRequest {
        nombre: None, // No puede cambiar nombre
        ruc: None, // No puede cambiar RUC
        direccion: request.direccion,
        telefono: request.telefono,
        correo: request.correo,
        encargado: None, // No puede cambiar encargado
        paleta_colores: request.paleta_colores,
        media: request.media, // Puede actualizar media (logo, etc.)
        is_active: None, // No puede cambiar estado
    };
    
    // Aplicar actualizaciones y guardar
    let updated = limited_request.apply_to(old_transporte.clone(), Some(auth.user.id));
    let result = state.container.transporte_repository.update(&updated).await?;
    
    // Logging del evento
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        transporte_id,
        Some(&old_transporte),
        Some(&result),
        None,
        None,
    ).await;
    
    info!("✅ Transporte actualizado por su usuario: {}", result.nombre);
    
    Ok(json_ok(TransporteResponse::from(result)))
}

/// Actualizar solo la interfaz de mi transporte (logo y paleta de colores)
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
    let mut transporte: Option<crate::domain::entities::Transporte> = None;
    
    // Verificar si el usuario tiene rol de transporte
    let is_transporte_user = auth.user.role == UserRole::Transportes;
    
    if is_transporte_user {
        if let Some(id_entidad) = auth.user.id_entidad {
            transporte = state.container.transporte_repository
                .find_by_id(id_entidad)
                .await?;
        }
    }
    
    if transporte.is_none() {
        if let Some(persona_id) = auth.user.id_persona {
            transporte = state.container.transporte_repository
                .find_by_encargado(persona_id)
                .await?;
        }
    }
    
    let old_transporte = transporte.ok_or_else(|| {
        ApplicationError::NotFound("No tienes un transporte asociado".to_string())
    })?;
    
    let transporte_id = old_transporte.id;
    
    // Aplicar cambios solo de interfaz
    let updated = request.apply_to(old_transporte.clone(), Some(auth.user.id));
    let result = state.container.transporte_repository.update(&updated).await?;
    
    // Logging
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        transporte_id,
        Some(&old_transporte),
        Some(&result),
        Some(vec!["paleta_colores".to_string(), "media".to_string()]),
        None,
    ).await;
    
    info!("✅ Interfaz de transporte '{}' actualizada", result.nombre);
    
    Ok(json_ok(TransporteResponse::from(result)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    if state.container.transporte_repository.exists_by_ruc(&request.ruc).await? {
        return Err(ApplicationError::Conflict(format!("RUC {} ya existe", request.ruc)));
    }
    let created = state.container.transporte_repository.create(&request.into_entity(Some(auth.user.id))).await?;
    info!("✅ Transporte creado: {} (ID: {})", created.nombre, created.id);

    // Log activity
    let _ = state.container.logging_service.log_create::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        created.id,
        &created.nombre,
        Some(&created),
        None,
    ).await;

    // Notify admins with SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Nuevo transporte creado",
        &format!("{} ha creado el transporte '{}'", auth.user.username, created.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_created(TransporteResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTransporteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_t = state.container.transporte_repository.find_by_id(id).await?.ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;
    let result = state.container.transporte_repository.update(&request.apply_to(old_t.clone(), Some(auth.user.id))).await?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        result.id,
        Some(&old_t),
        Some(&result),
        None,
        None,
    ).await;

    // Notify admins via SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Transporte actualizado",
        &format!("{} ha actualizado el transporte '{}'", auth.user.username, result.nombre),
        NotificationType::Info,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_ok(TransporteResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Get transporte info before deleting
    let transporte = state.container.transporte_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;

    if !state.container.transporte_repository.soft_delete(id, auth.user.id).await? {
        return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
    }

    // Log activity
    let _ = state.container.logging_service.log_delete::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        id,
        Some(&transporte),
        None,
    ).await;

    // Notify admins with SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Transporte eliminado",
        &format!("{} ha eliminado el transporte '{}'", auth.user.username, transporte.nombre),
        NotificationType::Warning,
        NotificationCategory::Crud,
        NotificationPriority::Normal,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Transporte desactivado"))
}

#[instrument(skip(state, auth))]
pub async fn restore_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !state.container.transporte_repository.restore(id, auth.user.id).await? {
        return Err(ApplicationError::NotFound(format!("Transporte {} no encontrado", id)));
    }

    // Get transporte info after restore
    let transporte = state.container.transporte_repository.find_by_id(id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Transporte {} no encontrado", id)))?;

    // Log activity
    let _ = state.container.logging_service.log_update::<crate::domain::entities::Transporte>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Transporte,
        id,
        None,
        Some(&transporte),
        Some(vec!["is_active".to_string()]),
        None,
    ).await;

    // Notify admins with SSE broadcast
    let _ = state.notify_roles_with_broadcast(
        vec![UserRole::SuperAdmin, UserRole::Admin],
        "Transporte restaurado",
        &format!("{} ha restaurado el transporte '{}'", auth.user.username, transporte.nombre),
        NotificationType::Success,
        NotificationCategory::Crud,
        NotificationPriority::Low,
        Some(auth.user.id),
    ).await;

    Ok(json_message("Transporte restaurado"))
}

#[instrument(skip(state, _auth))]
pub async fn list_transportes_with_vehicles(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let transportes = state.container.transporte_repository.find_with_available_vehicles().await?;
    let response: Vec<TransporteResponse> = transportes.into_iter().map(Into::into).collect();
    Ok(json_ok(response))
}
