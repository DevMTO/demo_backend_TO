//! Handlers para FileGuias (guías vinculados a file_tours)

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{instrument, info};
use validator::Validate;

use crate::application::dtos::{AssignGuiaToFileTourRequest, FileGuiaResponse, UpdateFileGuiaRequest};
use crate::domain::entities::StatusGuia;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};
use crate::infrastructure::persistence::models::file_guia_model::UpdateFileGuiaModel;

/// Lista los guías asignados a un file_tour con info completa de persona
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_guias(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    // Usar método con JOIN para obtener info completa de guía y persona
    let guias = state.container.file_guia_repository
        .find_by_file_tour_with_persona(file_tour_id)
        .await?;
    
    let responses: Vec<FileGuiaResponse> = guias.into_iter()
        .map(FileGuiaResponse::from)
        .collect();
    
    Ok(json_ok(responses))
}

/// Asigna un guía a un file_tour
#[instrument(skip(state, auth, request))]
pub async fn assign_guia_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignGuiaToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe
    let file_tour = state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    // Verificar que el guía existe
    let guia = state.container.guia_repository
        .find_by_id(request.id_guia)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", request.id_guia)))?;
    
    // Verificar que el guía no esté ya asignado a este file_tour
    if state.container.file_guia_repository
        .is_guia_assigned(request.id_guia, request.id_file_tour)
        .await? 
    {
        return Err(ApplicationError::Validation("El guía ya está asignado a este file_tour".to_string()));
    }
    
    let rol = request.rol.clone().unwrap_or_else(|| "Guía".to_string());
    
    // Asignar el guía
    let result = state.container.file_guia_repository
        .add(request.id_file_tour, request.id_guia, request.rol.as_deref(), Some(auth.user.id))
        .await?;
    
    // Cambiar el status del guía a "ocupado"
    if guia.status == StatusGuia::Disponible {
        state.container.guia_repository
            .update_status(request.id_guia, "en_servicio")
            .await?;
    }
    
    // ===== AUTO-CREAR PAGO PROVEEDOR (guia) =====
    let _ = state.container.contabilidad_service
        .auto_create_pago_proveedor(
            "guia",
            None,
            None,
            Some(request.id_guia),
            None,
            Some(request.id_file_tour),
            None,
            None,
            Some(result.id),
            None,
            None,
            Some(auth.user.id),
        ).await;
    
    // ===== NOTIFICAR AL GUÍA ASIGNADO =====
    // Obtener información del file para la notificación
    let file = state.container.file_repository
        .find_by_id(file_tour.id_file)
        .await?;
    
    if let Some(file) = file {
        // Obtener información del tour
        let tour = state.container.tour_repository
            .find_by_id(file_tour.id_tour)
            .await?;
        
        let tour_name = tour.map(|t| t.nombre.clone()).unwrap_or_else(|| "Tour".to_string());
        let file_code = file.file_code.clone().unwrap_or_else(|| format!("F-{}", file.id));
        let fecha = file.fecha_inicio.format("%d/%m/%Y").to_string();
        
        // Notificar al guía
        let _ = state.container.file_assignment_service
            .notify_guia_assignment(
                request.id_guia,
                &file_code,
                &tour_name,
                &fecha,
                &rol,
                Some(auth.user.id),
            ).await;
    }
    
    // ===== VERIFICAR SI EL FILE ESTÁ COMPLETAMENTE ASIGNADO =====
    let _ = state.container.file_assignment_service
        .check_and_update_file_status(file_tour.id_file, auth.user.id)
        .await;
    
    Ok(json_created(FileGuiaResponse::from(result)))
}

/// Elimina una asignación de guía
#[instrument(skip(state, _auth))]
pub async fn remove_file_guia(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(guia_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let _asig = state.container.file_guia_repository
        .find_by_id(guia_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    // Liberar el guía si ya no tiene más asignaciones
    state.container.file_guia_repository.remove(guia_asig_id).await?;
    
    // TODO: Verificar si el guía tiene otras asignaciones activas antes de cambiar su status
    
    Ok(json_deleted())
}

/// Actualiza un file_guia (cambiar guía, rol, file_tour, status)
#[instrument(skip(state, auth, request))]
pub async fn update_file_guia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateFileGuiaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que la asignación existe
    let existing = state.container.file_guia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Asignación file_guia {} no encontrada", id)))?;
    
    // Si se cambia el guía, verificar que existe y no está duplicado
    if let Some(new_guia_id) = request.id_guia {
        state.container.guia_repository
            .find_by_id(new_guia_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Guía {} no encontrado", new_guia_id)))?;
        
        // Verificar duplicado en el file_tour destino
        let target_ft = request.id_file_tour.unwrap_or(existing.id_file_tour);
        if state.container.file_guia_repository.is_guia_assigned(new_guia_id, target_ft).await? {
            return Err(ApplicationError::Validation("El guía ya está asignado a este file_tour".to_string()));
        }
    }
    
    // Si se cambia el file_tour, verificar que existe
    if let Some(new_ft_id) = request.id_file_tour {
        state.container.file_tour_repository
            .find_by_id(new_ft_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", new_ft_id)))?;
        
        // Verificar duplicado si no cambia el guía
        if request.id_guia.is_none()
            && state.container.file_guia_repository.is_guia_assigned(existing.id_guia, new_ft_id).await?
        {
            return Err(ApplicationError::Validation("El guía ya está asignado a este file_tour".to_string()));
        }
    }
    
    // Manejar cambio de rol (Option<Option<String>> para poder setear a NULL)
    let rol_change: Option<Option<String>> = if request.clear_rol {
        Some(None)
    } else {
        request.rol.map(Some)
    };
    
    let update_data = UpdateFileGuiaModel {
        id_guia: request.id_guia,
        rol: rol_change,
        id_file_tour: request.id_file_tour,
        status: request.status.clone(),
    };
    
    let result = state.container.file_guia_repository
        .update(id, update_data)
        .await?;
    
    info!("FileGuia {} actualizado por usuario {}", id, auth.user.id);
    
    Ok(json_ok(FileGuiaResponse::from(result)))
}
