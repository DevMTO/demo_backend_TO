//! Handlers para FilePasajeros (pasajeros vinculados a files)

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use chrono::Datelike;
use tracing::{instrument, info};
use validator::Validate;

use crate::application::dtos::{
    AddPasajeroToFileRequest, FilePasajeroResponse, BulkAddPasajerosRequest,
    UpdateFilePasajeroRequest, CreatePasajeroWithPersonaRequest,
    CreatePasajeroWithPersonaResponse,
};
use crate::domain::entities::{Persona, TipoDocumento};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::file_pasajero_model::UpdateFilePasajeroModel;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};
use crate::application::dtos::FileRelationStatus;

/// Lista los pasajeros de un file
#[instrument(skip(state, _auth))]
pub async fn list_file_pasajeros(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    let pasajeros = state.container.file_pasajero_repository
        .find_by_file_with_persona(file_id)
        .await?;
    
    let responses: Vec<FilePasajeroResponse> = pasajeros.into_iter()
        .map(FilePasajeroResponse::from)
        .collect();
    
    Ok(json_ok(responses))
}

/// Agrega un pasajero a un file
/// - id_persona es opcional para permitir pasajeros anónimos
/// - edad es opcional
#[instrument(skip(state, auth, request))]
pub async fn add_pasajero_to_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
    Json(request): Json<AddPasajeroToFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file existe
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    // Si se proporciona id_persona, verificar que existe
    if let Some(persona_id) = request.id_persona {
        state.container.persona_repository
            .find_by_id(persona_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Persona {} no encontrada", persona_id)))?;
    }
    
    let result = state.container.file_pasajero_repository
        .add(
            file_id, 
            request.id_persona, 
            request.asiento.as_deref(),
            request.tipo_pasajero.as_deref(),
            request.nacionalidad.as_deref(),
            request.notas.as_deref(),
            request.edad,
            Some(auth.user.id),
        )
        .await?;
    
    // NOTA: nro_pasajeros es un valor fijo contratado, no se actualiza al agregar pasajeros
    
    Ok(json_created(FilePasajeroResponse::from(result)))
}

/// Agrega múltiples pasajeros a un file (bulk import desde Excel)
/// NOTA: nro_pasajeros es un valor fijo contratado y no se modifica
#[instrument(skip(state, auth, request))]
pub async fn bulk_add_pasajeros_to_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
    Json(request): Json<BulkAddPasajerosRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el file existe
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    let mut results: Vec<FilePasajeroResponse> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    
    for (idx, pasajero) in request.pasajeros.iter().enumerate() {
        // Validar cada pasajero
        if let Err(e) = pasajero.validate() {
            errors.push(format!("Pasajero {}: {}", idx + 1, e));
            continue;
        }
        
        // Si se proporciona id_persona, verificar que existe
        if let Some(persona_id) = pasajero.id_persona {
            if state.container.persona_repository.find_by_id(persona_id).await?.is_none() {
                errors.push(format!("Pasajero {}: Persona {} no encontrada", idx + 1, persona_id));
                continue;
            }
        }
        
        match state.container.file_pasajero_repository
            .add(
                file_id, 
                pasajero.id_persona, 
                pasajero.asiento.as_deref(),
                pasajero.tipo_pasajero.as_deref(),
                pasajero.nacionalidad.as_deref(),
                pasajero.notas.as_deref(),
                pasajero.edad,
                Some(auth.user.id),
            )
            .await
        {
            Ok(result) => results.push(FilePasajeroResponse::from(result)),
            Err(e) => errors.push(format!("Pasajero {}: {}", idx + 1, e)),
        }
    }
    
    // NOTA: nro_pasajeros es un valor fijo contratado, NO se actualiza al agregar pasajeros
    // Solo contamos cuántos pasajeros se asignaron para informar al usuario
    let current_count = state.container.file_pasajero_repository
        .count_by_file(file_id)
        .await? as i32;
    
    info!("Bulk import: {} pasajeros agregados al file {}, total asignados: {}", results.len(), file_id, current_count);
    
    #[derive(serde::Serialize)]
    struct BulkAddResponse {
        success: bool,
        added_count: usize,
        total_asignados: i32,  // Renombrado para claridad: es el conteo de pasajeros asignados, no nro_pasajeros contratado
        errors: Vec<String>,
        pasajeros: Vec<FilePasajeroResponse>,
    }
    
    Ok(json_ok(BulkAddResponse {
        success: errors.is_empty(),
        added_count: results.len(),
        total_asignados: current_count,
        errors,
        pasajeros: results,
    }))
}

/// Elimina un pasajero de un file
#[instrument(skip(state, _auth))]
pub async fn remove_file_pasajero(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((file_id, pasajero_asig_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, ApplicationError> {
    let asig = state.container.file_pasajero_repository
        .find_by_id(pasajero_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Pasajero no encontrado en este file".to_string()))?;
    
    if asig.id_file != file_id {
        return Err(ApplicationError::Validation("El pasajero no pertenece a este file".to_string()));
    }
    
    state.container.file_pasajero_repository.remove(pasajero_asig_id).await?;
    
    // NOTA: nro_pasajeros es un valor fijo contratado, no se modifica al eliminar pasajeros
    
    Ok(json_deleted())
}

/// Actualiza la información de un pasajero en el file
#[instrument(skip(state, _auth, request))]
pub async fn update_file_pasajero(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((file_id, pasajero_id)): Path<(i32, i32)>,
    Json(request): Json<UpdateFilePasajeroRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el pasajero existe y pertenece al file
    let existing = state.container.file_pasajero_repository
        .find_by_id(pasajero_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Pasajero {} no encontrado", pasajero_id)))?;
    
    if existing.id_file != file_id {
        return Err(ApplicationError::Validation("El pasajero no pertenece a este file".to_string()));
    }
    
    // Validar status si se proporciona
    if let Some(ref status) = request.status {
        FileRelationStatus::from_str(status)
            .map_err(|e| ApplicationError::Validation(e))?;
    }
    
    // Construir modelo de actualización
    let update_data = UpdateFilePasajeroModel {
        id_persona: request.id_persona.map(Some), // Convierte Option<i32> a Option<Option<i32>>
        asiento: request.asiento.clone(),
        tipo_pasajero: request.tipo_pasajero.clone(),
        notas: request.notas.clone(),
        nacionalidad: request.nacionalidad.clone(),
        edad: request.edad,
        status: request.status.clone(),
    };
    
    let result = state.container.file_pasajero_repository
        .update(pasajero_id, update_data)
        .await?;
    
    info!("Pasajero {} actualizado en file {}", pasajero_id, file_id);
    
    Ok(json_ok(FilePasajeroResponse::from(result)))
}

/// Crea un pasajero con persona (si no existe la persona, la crea)
/// Este endpoint permite agregar un pasajero al file creando también los datos de persona
#[instrument(skip(state, auth, request))]
pub async fn create_pasajero_with_persona(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
    Json(request): Json<CreatePasajeroWithPersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file existe
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    // Buscar si ya existe una persona con ese documento
    let existing_persona = state.container.persona_repository
        .find_by_documento(&request.tipo_documento, &request.nro_documento)
        .await?;
    
    let (persona_id, persona_nombre, persona_apellidos, persona_documento, persona_created) = 
        if let Some(persona) = existing_persona {
            // La persona ya existe
            (persona.id, persona.nombre, persona.apellidos, persona.nro_documento, false)
        } else {
            // Crear la persona
            let tipo_doc = request.tipo_documento.parse::<TipoDocumento>()
                .unwrap_or(TipoDocumento::Dni);
            
            let mut new_persona = Persona::new(
                tipo_doc,
                request.nro_documento.clone(),
                request.nombre.clone(),
                request.apellidos.clone(),
            );
            // Asignar campos opcionales
            new_persona.telefono = request.telefono.clone();
            new_persona.correo = request.correo.clone();
            new_persona.fecha_nacimiento = request.fecha_nacimiento;
            new_persona.created_by = Some(auth.user.id);
            new_persona.updated_by = Some(auth.user.id);
            
            let created = state.container.persona_repository
                .create(&new_persona)
                .await?;
            
            (created.id, created.nombre, created.apellidos, created.nro_documento, true)
        };
    
    // Agregar como pasajero al file
    // Nota: Este endpoint siempre crea/usa una persona, así que id_persona no es None
    // edad se calcula de fecha_nacimiento si está disponible
    let edad = request.fecha_nacimiento.map(|fecha| {
        let hoy = chrono::Utc::now().date_naive();
        let mut age = hoy.year() - fecha.year();
        if (hoy.month(), hoy.day()) < (fecha.month(), fecha.day()) {
            age -= 1;
        }
        age
    });
    
    let pasajero_result = state.container.file_pasajero_repository
        .add(
            file_id,
            Some(persona_id),  // Siempre tiene persona en este endpoint
            request.asiento.as_deref(),
            request.tipo_pasajero.as_deref(),
            request.nacionalidad.as_deref(),
            request.notas.as_deref(),
            edad,
            Some(auth.user.id),
        )
        .await?;
    
    // NOTA: nro_pasajeros es un valor fijo contratado, no se actualiza al agregar pasajeros
    
    let mut pasajero_response = FilePasajeroResponse::from(pasajero_result);
    pasajero_response.pasajero_nombre = Some(persona_nombre.clone());
    pasajero_response.pasajero_apellidos = Some(persona_apellidos.clone());
    pasajero_response.pasajero_documento = Some(persona_documento.clone());
    
    let response = CreatePasajeroWithPersonaResponse {
        persona_id,
        persona_nombre,
        persona_apellidos,
        persona_documento,
        pasajero_asignacion: pasajero_response,
        persona_created,
    };
    
    Ok(json_created(response))
}
