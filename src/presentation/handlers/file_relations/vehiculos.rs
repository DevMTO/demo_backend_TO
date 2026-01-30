//! Handlers para FileVehiculos (vehículos vinculados a file_tours)

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{
    AssignVehiculoToFileTourRequest, FileVehiculoResponse, FileVehiculoListItemDto,
    ResourceStatusUpdateResponse, UpdateVehiculoStatusRequest,
};
use crate::domain::entities::StatusConductor;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};

/// Lista TODOS los file_vehiculos con información completa
#[instrument(skip(state, _auth))]
pub async fn list_all_file_vehiculos(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let vehiculos = state.container.file_vehiculo_repository
        .find_all_with_details()
        .await?;
    
    let responses: Vec<FileVehiculoListItemDto> = vehiculos.into_iter()
        .map(|v| FileVehiculoListItemDto {
            id: v.id,
            id_file_tour: v.id_file_tour,
            id_vehiculo: v.id_vehiculo,
            id_conductor: v.id_conductor,
            created_at: v.created_at,
            capacidad_asignada: v.capacidad_asignada,
            status: v.status,
            file_code: v.file_code,
            file_fecha_inicio: v.file_fecha_inicio,
            file_fecha_fin: v.file_fecha_fin,
            file_status: v.file_status,
            file_nro_pasajeros: v.file_nro_pasajeros,
            tour_id: v.tour_id,
            tour_nombre: v.tour_nombre,
            agencia_id: v.agencia_id,
            agencia_nombre: v.agencia_nombre,
            vehiculo_nombre: v.vehiculo_nombre,
            vehiculo_placa: v.vehiculo_placa,
            vehiculo_capacidad: v.vehiculo_capacidad,
            conductor_nombre: v.conductor_nombre,
            conductor_brevete: v.conductor_brevete,
        })
        .collect();
    
    Ok(json_ok(responses))
}

/// Lista los vehículos asignados a un file_tour con info completa
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_vehiculos(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    // Usar método con JOIN para obtener info completa de vehículo, transporte y conductor
    let vehiculos = state.container.file_vehiculo_repository
        .find_by_file_tour_with_persona(file_tour_id)
        .await?;
    
    let responses: Vec<FileVehiculoResponse> = vehiculos.into_iter()
        .map(FileVehiculoResponse::from)
        .collect();
    
    Ok(json_ok(responses))
}

/// Asigna un vehículo a un file_tour
#[instrument(skip(state, auth, request))]
pub async fn assign_vehiculo_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignVehiculoToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe y obtener file_id para contar pasajeros
    let file_tour = state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    // Verificar que el vehículo existe
    let vehiculo = state.container.vehiculo_repository
        .find_by_id(request.id_vehiculo)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", request.id_vehiculo)))?;
    
    // Verificar que el vehículo no esté ya asignado a este file_tour
    if state.container.file_vehiculo_repository
        .is_vehiculo_assigned(request.id_vehiculo, request.id_file_tour)
        .await? 
    {
        return Err(ApplicationError::Validation("El vehículo ya está asignado a este file_tour".to_string()));
    }
    
    // Si se especificó un conductor, verificar que existe y está disponible
    if let Some(id_conductor) = request.id_conductor {
        let conductor = state.container.conductor_repository
            .find_by_id(id_conductor)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Conductor {} no encontrado", id_conductor)))?;
        
        // Cambiar status del conductor a ocupado
        if conductor.status == StatusConductor::Disponible {
            state.container.conductor_repository
                .update_status(id_conductor, "en_servicio")
                .await?;
        }
    }
    
    // Determinar capacidad asignada (por defecto toda la capacidad del vehículo)
    let capacidad_asignada = request.capacidad_asignada.unwrap_or(vehiculo.capacidad);
    
    // Asignar el vehículo
    let result = state.container.file_vehiculo_repository
        .add(request.id_file_tour, request.id_vehiculo, request.id_conductor, capacidad_asignada, Some(auth.user.id))
        .await?;
    
    // Contar pasajeros actuales del file para verificar capacidad
    let pax_count = state.container.file_pasajero_repository
        .count_by_file(file_tour.id_file)
        .await? as i32;
    
    // Si los pasajeros llenan o superan la capacidad, marcar como ocupado
    if pax_count >= vehiculo.capacidad {
        state.container.vehiculo_repository
            .update_status(request.id_vehiculo, "ocupado")
            .await?;
    }
    
    Ok(json_created(FileVehiculoResponse::from(result)))
}

/// Elimina una asignación de vehículo
#[instrument(skip(state, _auth))]
pub async fn remove_file_vehiculo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(vehiculo_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let asig = state.container.file_vehiculo_repository
        .find_by_id(vehiculo_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    // Liberar conductor si estaba asignado
    if let Some(id_conductor) = asig.id_conductor {
        // TODO: Verificar si el conductor tiene otras asignaciones activas
        state.container.conductor_repository
            .update_status(id_conductor, "disponible")
            .await?;
    }
    
    // Verificar si el vehículo tiene otras asignaciones activas
    let other_files = state.container.file_vehiculo_repository
        .find_files_by_vehiculo(asig.id_vehiculo)
        .await?;
    
    // Si solo tiene esta asignación (que vamos a eliminar), liberar el vehículo
    if other_files.len() <= 1 {
        state.container.vehiculo_repository
            .update_status(asig.id_vehiculo, "disponible")
            .await?;
    }
    
    state.container.file_vehiculo_repository.remove(vehiculo_asig_id).await?;
    Ok(json_deleted())
}

/// Cambia manualmente el status de un vehículo asignado a un file_tour
#[instrument(skip(state, _auth))]
pub async fn update_vehiculo_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((file_tour_id, vehiculo_id)): Path<(i32, i32)>,
    Json(request): Json<UpdateVehiculoStatusRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el vehículo está asignado a este file_tour
    if !state.container.file_vehiculo_repository
        .is_vehiculo_assigned(vehiculo_id, file_tour_id)
        .await? 
    {
        return Err(ApplicationError::Validation("El vehículo no está asignado a este file_tour".to_string()));
    }
    
    // Obtener status actual
    let vehiculo = state.container.vehiculo_repository
        .find_by_id(vehiculo_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Vehículo {} no encontrado", vehiculo_id)))?;
    
    let old_status = vehiculo.status.to_string();
    
    // Actualizar status
    state.container.vehiculo_repository
        .update_status(vehiculo_id, &request.status)
        .await?;
    
    Ok(json_ok(ResourceStatusUpdateResponse {
        resource_type: "vehiculo".to_string(),
        resource_id: vehiculo_id,
        old_status,
        new_status: request.status,
        message: "Status actualizado correctamente".to_string(),
    }))
}
