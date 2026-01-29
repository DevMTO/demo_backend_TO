use axum::{extract::{Path, State}, response::IntoResponse, Json};
use chrono::Datelike;
use tracing::{instrument, info};
use validator::Validate;

use crate::application::dtos::{
    AssignEntradaToFileTourRequest, AssignGuiaToFileTourRequest, AddPasajeroToFileRequest,
    AssignRestauranteToFileTourRequest, AssignVehiculoToFileTourRequest,
    FileEntradaResponse, FileGuiaResponse, FilePasajeroResponse,
    FileRestauranteResponse, FileVehiculoResponse, FileVehiculoListItemDto,
    ResourceStatusUpdateResponse, FileTourDto, BulkAddPasajerosRequest,
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::{StatusGuia, StatusConductor};
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{json_ok, json_created, json_deleted};

// ==================== FILE ENTRADAS (vinculadas a file_tours) ====================

/// Lista las entradas asignadas a un file_tour
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_entradas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el file_tour existe
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    let entradas = state.container.file_entrada_repository
        .find_by_file_tour(file_tour_id)
        .await?;
    
    // Obtener información completa de cada entrada
    let mut responses: Vec<FileEntradaResponse> = Vec::new();
    for e in entradas {
        let mut response = FileEntradaResponse::from(e.clone());
        if let Ok(Some(entrada)) = state.container.entrada_repository.find_by_id(e.id_entrada).await {
            response.entrada_nombre = Some(entrada.nombre);
            // El precio ahora se obtiene de entrada_precios
            response.entrada_precio = None;
        }
        responses.push(response);
    }
    
    Ok(json_ok(responses))
}

/// Asigna una entrada a un file_tour
#[instrument(skip(state, auth, request))]
pub async fn assign_entrada_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignEntradaToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe
    state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    // Verificar que la entrada existe
    state.container.entrada_repository
        .find_by_id(request.id_entrada)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", request.id_entrada)))?;
    
    let result = state.container.file_entrada_repository
        .add(request.id_file_tour, request.id_entrada, request.cantidad, request.id_entrada_precio, Some(auth.user.id))
        .await?;
    
    Ok(json_created(FileEntradaResponse::from(result)))
}

/// Elimina una asignación de entrada
#[instrument(skip(state, _auth))]
pub async fn remove_file_entrada(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(entrada_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe la asignación
    state.container.file_entrada_repository
        .find_by_id(entrada_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    state.container.file_entrada_repository.remove(entrada_asig_id).await?;
    Ok(json_deleted())
}

// ==================== FILE GUIAS (vinculadas a file_tours) ====================

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
    state.container.file_tour_repository
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

// ==================== FILE TOURS ====================

/// Lista los tours asignados a un file
#[instrument(skip(state, _auth))]
pub async fn list_file_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el file existe
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    // Obtener tours con información completa (INNER JOIN)
    let tours = state.container.file_tour_repository
        .find_by_file_with_tour(file_id)
        .await?;
    
    // Convertir a DTOs
    let responses: Vec<FileTourDto> = tours.into_iter()
        .map(|t| FileTourDto {
            id: t.id,
            id_tour: t.id_tour,
            orden: t.orden,
            precio_aplicado: t.precio_aplicado.clone(),
            notas: t.notas,
            fecha_tour: t.fecha_tour,
            turno_tour: t.turno_tour,
            lugar_recojo: t.lugar_recojo,
            hora_recojo: t.hora_recojo,
            status: t.status,
            tour_nombre: Some(t.tour_nombre),
            tour_lugar_inicio: Some(t.tour_lugar_inicio),
            tour_lugar_fin: Some(t.tour_lugar_fin),
            tour_precio_base: Some(t.tour_precio_base),
            tour_duracion_dias: t.tour_duracion_dias,
            tour_tipo: t.tour_tipo,
            tour_is_active: Some(t.tour_is_active),
        })
        .collect();
    
    Ok(json_ok(responses))
}

// ==================== FILE PASAJEROS ====================

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
    Json(request): Json<crate::application::dtos::UpdateFilePasajeroRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    use crate::infrastructure::persistence::models::file_pasajero_model::UpdateFilePasajeroModel;
    
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
        crate::application::dtos::FileRelationStatus::from_str(status)
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
    Json(request): Json<crate::application::dtos::CreatePasajeroWithPersonaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    use crate::application::dtos::{CreatePasajeroWithPersonaResponse, FilePasajeroResponse};
    use crate::domain::entities::{Persona, TipoDocumento};
    
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

// ==================== FILE RESTAURANTES (vinculados a file_tours) ====================

/// Lista los restaurantes asignados a un file_tour
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_restaurantes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    let restaurantes = state.container.file_restaurante_repository
        .find_by_file_tour(file_tour_id)
        .await?;
    
    // Obtener información completa de cada restaurante
    let mut responses: Vec<FileRestauranteResponse> = Vec::new();
    for r in restaurantes {
        let mut response = FileRestauranteResponse::from(r.clone());
        if let Ok(Some(restaurante)) = state.container.restaurante_repository.find_by_id(r.id_restaurante).await {
            response.restaurante_nombre = Some(restaurante.nombre);
            response.restaurante_direccion = Some(restaurante.direccion);
        }
        responses.push(response);
    }
    
    Ok(json_ok(responses))
}

/// Asigna un restaurante a un file_tour específico
#[instrument(skip(state, auth, request))]
pub async fn assign_restaurante_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignRestauranteToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe
    state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    state.container.restaurante_repository
        .find_by_id(request.id_restaurante)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", request.id_restaurante)))?;
    
    // Convertir precio de f64 a BigDecimal si se proporciona
    let precio = request.precio.map(|p| bigdecimal::BigDecimal::try_from(p).unwrap_or_default());
    
    let result = state.container.file_restaurante_repository
        .add(
            request.id_file_tour, 
            request.id_restaurante, 
            request.tipo_servicio.as_deref(),
            precio,
            Some(auth.user.id),
        )
        .await?;
    
    Ok(json_created(FileRestauranteResponse::from(result)))
}

/// Elimina una asignación de restaurante
#[instrument(skip(state, _auth))]
pub async fn remove_file_restaurante(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(restaurante_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_restaurante_repository
        .find_by_id(restaurante_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    state.container.file_restaurante_repository.remove(restaurante_asig_id).await?;
    Ok(json_deleted())
}

// ==================== FILE VEHICULOS (vinculados a file_tours) ====================

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

// ==================== GESTIÓN DE STATUS ====================

/// Cambia manualmente el status de un vehículo asignado a un file_tour
#[instrument(skip(state, _auth))]
pub async fn update_vehiculo_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((file_tour_id, vehiculo_id)): Path<(i32, i32)>,
    Json(request): Json<crate::application::dtos::UpdateVehiculoStatusRequest>,
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
