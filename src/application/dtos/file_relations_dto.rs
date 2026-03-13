use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::infrastructure::persistence::models::{
    FileEntradaModel, FileGuiaModel, FileGuiaWithPersonaModel, FilePasajeroModel,
    FilePasajeroWithPersonaModel, FileRestauranteModel, FileVehiculoModel,
    FileVehiculoWithPersonaModel,
};

// ==================== FILE ENTRADA ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileEntradaResponse {
    pub id: i32,
    /// Referencia al file_tour específico
    pub id_file_tour: i32,
    pub id_entrada: i32,
    pub cantidad: i32,
    /// Referencia al precio específico elegido (opcional)
    pub id_entrada_precio: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Estado: reservado, confirmado, cancelado
    pub status: String,
    /// Historial de file_tours anteriores (transferencias BT)
    pub cancelaciones: Vec<i32>,
    // Datos de la entrada relacionada (se pueden poblar en el handler)
    pub entrada_nombre: Option<String>,
    pub entrada_precio: Option<String>,
}

impl From<FileEntradaModel> for FileEntradaResponse {
    fn from(m: FileEntradaModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_entrada: m.id_entrada,
            cantidad: m.cantidad,
            id_entrada_precio: m.id_entrada_precio,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            cancelaciones: m.cancelaciones.into_iter().flatten().collect(),
            entrada_nombre: None,
            entrada_precio: None,
        }
    }
}

/// Request para asignar entrada a un file_tour específico
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignEntradaToFileTourRequest {
    /// ID del file_tour al que se asigna la entrada
    pub id_file_tour: i32,
    pub id_entrada: i32,
    #[validate(range(min = 1, message = "La cantidad debe ser al menos 1"))]
    pub cantidad: i32,
    /// ID del precio específico para esta entrada (opcional)
    pub id_entrada_precio: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct BulkAssignEntradasToFileTourRequest {
    pub entradas: Vec<AssignEntradaToFileTourRequest>,
}

// ==================== FILE GUIA ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileGuiaResponse {
    pub id: i32,
    /// Referencia al file_tour específico
    pub id_file_tour: i32,
    pub id_guia: i32,
    pub rol: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Estado: pendiente (no aceptó), reservado (aceptó), confirmado, cancelado
    pub status: String,
    /// Estado de confirmación: pendiente, confirmado, rechazado
    pub estado_confirmacion: Option<String>,
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<String>,
    // Datos del guía relacionado (de tabla guias)
    pub guia_nro_carnet: Option<String>,
    pub guia_idiomas: Option<String>,
    // Datos de la persona asociada al guía (de tabla personas)
    pub guia_nombre: Option<String>,
    pub guia_apellidos: Option<String>,
    pub guia_telefono: Option<String>,
    pub guia_correo: Option<String>,
}

impl From<FileGuiaModel> for FileGuiaResponse {
    fn from(m: FileGuiaModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_guia: m.id_guia,
            rol: m.rol,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            estado_confirmacion: Some(m.estado_confirmacion),
            confirmado_at: m.confirmado_at,
            motivo_rechazo: m.motivo_rechazo,
            guia_nro_carnet: None,
            guia_idiomas: None,
            guia_nombre: None,
            guia_apellidos: None,
            guia_telefono: None,
            guia_correo: None,
        }
    }
}

impl From<FileGuiaWithPersonaModel> for FileGuiaResponse {
    fn from(m: FileGuiaWithPersonaModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_guia: m.id_guia,
            rol: m.rol,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            estado_confirmacion: Some(m.estado_confirmacion),
            confirmado_at: m.confirmado_at,
            motivo_rechazo: m.motivo_rechazo,
            guia_nro_carnet: m.guia_nro_carnet,
            guia_idiomas: m.guia_idiomas,
            guia_nombre: m.guia_nombre,
            guia_apellidos: m.guia_apellidos,
            guia_telefono: m.guia_telefono,
            guia_correo: m.guia_correo,
        }
    }
}

/// Request para asignar guía a un file_tour específico
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignGuiaToFileTourRequest {
    /// ID del file_tour al que se asigna el guía
    pub id_file_tour: i32,
    pub id_guia: i32,
    #[validate(length(max = 30))]
    pub rol: Option<String>, // "principal", "auxiliar", etc.
}

// ==================== FILE PASAJERO ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FilePasajeroResponse {
    pub id: i32,
    pub id_file: i32,
    /// ID de persona (opcional para pasajeros anónimos)
    pub id_persona: Option<i32>,
    pub asiento: Option<String>,
    pub tipo_pasajero: Option<String>,
    pub notas: Option<String>,
    pub nacionalidad: Option<String>,
    /// Edad del pasajero al momento del viaje
    pub edad: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Estado: reservado, confirmado, no_show, cancelado
    pub status: String,
    // Datos del pasajero relacionado (pueden ser None si id_persona es None)
    pub pasajero_nombre: Option<String>,
    pub pasajero_apellidos: Option<String>,
    pub pasajero_tipo_documento: Option<String>,
    pub pasajero_documento: Option<String>,
    pub pasajero_telefono: Option<String>,
}

impl From<FilePasajeroModel> for FilePasajeroResponse {
    fn from(m: FilePasajeroModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_persona: m.id_persona,
            asiento: m.asiento,
            tipo_pasajero: m.tipo_pasajero,
            notas: m.notas,
            nacionalidad: m.nacionalidad,
            edad: m.edad,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            pasajero_nombre: None,
            pasajero_apellidos: None,
            pasajero_tipo_documento: None,
            pasajero_documento: None,
            pasajero_telefono: None,
        }
    }
}

impl From<FilePasajeroWithPersonaModel> for FilePasajeroResponse {
    fn from(m: FilePasajeroWithPersonaModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_persona: m.id_persona,
            asiento: m.asiento,
            tipo_pasajero: m.tipo_pasajero,
            notas: m.notas,
            nacionalidad: m.nacionalidad,
            edad: m.edad,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            pasajero_nombre: m.pasajero_nombre,
            pasajero_apellidos: m.pasajero_apellidos,
            pasajero_tipo_documento: m.pasajero_tipo_documento,
            pasajero_documento: m.pasajero_documento,
            pasajero_telefono: m.pasajero_telefono,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AddPasajeroToFileRequest {
    /// ID de persona (opcional para pasajeros anónimos)
    pub id_persona: Option<i32>,
    #[validate(length(max = 10))]
    pub asiento: Option<String>,
    #[validate(length(max = 30))]
    pub tipo_pasajero: Option<String>, // "adulto", "niño", "infante", "tercera_edad"
    #[validate(length(max = 60))]
    pub nacionalidad: Option<String>,
    /// Edad del pasajero al momento del viaje
    #[validate(range(min = 0, max = 120))]
    pub edad: Option<i32>,
    pub notas: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct BulkAddPasajerosRequest {
    pub pasajeros: Vec<AddPasajeroToFileRequest>,
}

/// DTO para crear pasajero con persona (si no existe)
/// Permite crear la persona y asignarla como pasajero en una sola operación
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePasajeroWithPersonaRequest {
    // Datos de la persona
    #[validate(length(min = 2, max = 30, message = "Tipo documento inválido"))]
    pub tipo_documento: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Nro documento debe tener entre 6 y 20 caracteres"
    ))]
    pub nro_documento: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "Nombre debe tener entre 2 y 100 caracteres"
    ))]
    pub nombre: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "Apellidos debe tener entre 2 y 100 caracteres"
    ))]
    pub apellidos: String,

    #[validate(length(max = 20, message = "Teléfono muy largo"))]
    pub telefono: Option<String>,

    #[validate(email(message = "Correo inválido"))]
    pub correo: Option<String>,

    pub fecha_nacimiento: Option<chrono::NaiveDate>,

    // Datos específicos de pasajero
    #[validate(length(max = 10))]
    pub asiento: Option<String>,

    #[validate(length(max = 30))]
    pub tipo_pasajero: Option<String>, // "adulto", "niño", "infante"

    #[validate(length(max = 60))]
    pub nacionalidad: Option<String>,

    pub notas: Option<String>,
}

/// Respuesta que incluye tanto la persona creada/encontrada como la asignación de pasajero
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePasajeroWithPersonaResponse {
    pub persona_id: i32,
    pub persona_nombre: String,
    pub persona_apellidos: String,
    pub persona_documento: String,
    pub pasajero_asignacion: FilePasajeroResponse,
    pub persona_created: bool, // true si se creó, false si ya existía
}

/// Request para actualizar información de un pasajero en el file
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFilePasajeroRequest {
    /// ID de persona (opcional para pasajeros anónimos)
    pub id_persona: Option<i32>,
    #[validate(length(max = 10))]
    pub asiento: Option<String>,
    #[validate(length(max = 30))]
    pub tipo_pasajero: Option<String>, // "adulto", "niño", "infante", "tercera_edad"
    #[validate(length(max = 60))]
    pub nacionalidad: Option<String>,
    /// Edad del pasajero al momento del viaje
    #[validate(range(min = 0, max = 120))]
    pub edad: Option<i32>,
    pub notas: Option<String>,
    /// Estado: pendiente, reservado, asignado, confirmado, en_curso, completado, cancelado, anulado
    #[validate(length(max = 20))]
    pub status: Option<String>,
}

// ==================== FILE RESTAURANTE ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileRestauranteResponse {
    pub id: i32,
    /// Referencia al file_tour específico
    pub id_file_tour: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Precio del servicio de restaurante para este tour
    #[ts(type = "string | null")]
    pub precio: Option<BigDecimal>,
    /// Estado: reservado, confirmado, cancelado
    pub status: String,
    // Datos del restaurante relacionado
    pub restaurante_nombre: Option<String>,
    pub restaurante_direccion: Option<String>,
}

impl From<FileRestauranteModel> for FileRestauranteResponse {
    fn from(m: FileRestauranteModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_restaurante: m.id_restaurante,
            tipo_servicio: m.tipo_servicio,
            created_at: m.created_at,
            created_by: m.created_by,
            precio: m.precio,
            status: m.status,
            restaurante_nombre: None,
            restaurante_direccion: None,
        }
    }
}

/// Request para asignar restaurante a un file_tour específico
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignRestauranteToFileTourRequest {
    /// ID del file_tour al que se asigna el restaurante
    pub id_file_tour: i32,
    pub id_restaurante: i32,
    #[validate(length(max = 30))]
    pub tipo_servicio: Option<String>, // "desayuno", "almuerzo", "cena"
    /// Precio del servicio de restaurante
    #[ts(type = "number | null")]
    pub precio: Option<f64>,
}

// ==================== FILE VEHICULO ====================

/// DTO extendido para listar todos los file_vehiculos con información completa
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileVehiculoListItemDto {
    pub id: i32,
    /// Referencia al file_tour específico
    pub id_file_tour: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub capacidad_asignada: i32,
    /// Estado de la asignación: reservado, confirmado, cancelado
    pub status: String,
    // Datos del file
    pub file_code: Option<String>,
    pub file_fecha_inicio: String,
    pub file_fecha_fin: String,
    pub file_status: String,
    pub file_nro_pasajeros: i32,
    // Datos del tour
    pub tour_id: i32,
    pub tour_nombre: String,
    // Datos de la agencia
    pub agencia_id: i32,
    pub agencia_nombre: String,
    // Datos del vehículo
    pub vehiculo_nombre: Option<String>,
    pub vehiculo_placa: Option<String>,
    pub vehiculo_capacidad: Option<i32>,
    // Datos del conductor (si tiene)
    pub conductor_nombre: Option<String>,
    pub conductor_brevete: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileVehiculoResponse {
    pub id: i32,
    /// Referencia al file_tour específico
    pub id_file_tour: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub capacidad_asignada: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    /// Estado: reservado, confirmado, cancelado
    pub status: String,
    // Datos del vehículo
    pub vehiculo_nombre: Option<String>,
    pub vehiculo_placa: Option<String>,
    pub vehiculo_capacidad: Option<i32>,
    pub vehiculo_modelo: Option<String>,
    // Datos del transporte (empresa)
    pub transporte_id: Option<i32>,
    pub transporte_nombre: Option<String>,
    pub transporte_ruc: Option<String>,
    pub transporte_telefono: Option<String>,
    // Datos del conductor
    pub conductor_brevete: Option<String>,
    pub conductor_nombre: Option<String>,
    pub conductor_apellidos: Option<String>,
    pub conductor_telefono: Option<String>,
}

impl From<FileVehiculoModel> for FileVehiculoResponse {
    fn from(m: FileVehiculoModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_vehiculo: m.id_vehiculo,
            id_conductor: m.id_conductor,
            capacidad_asignada: m.capacidad_asignada,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            vehiculo_nombre: None,
            vehiculo_placa: None,
            vehiculo_capacidad: None,
            vehiculo_modelo: None,
            transporte_id: None,
            transporte_nombre: None,
            transporte_ruc: None,
            transporte_telefono: None,
            conductor_brevete: None,
            conductor_nombre: None,
            conductor_apellidos: None,
            conductor_telefono: None,
        }
    }
}

impl From<FileVehiculoWithPersonaModel> for FileVehiculoResponse {
    fn from(m: FileVehiculoWithPersonaModel) -> Self {
        Self {
            id: m.id,
            id_file_tour: m.id_file_tour,
            id_vehiculo: m.id_vehiculo,
            id_conductor: m.id_conductor,
            capacidad_asignada: m.capacidad_asignada,
            created_at: m.created_at,
            created_by: m.created_by,
            status: m.status,
            vehiculo_nombre: m.vehiculo_nombre,
            vehiculo_placa: m.vehiculo_placa,
            vehiculo_capacidad: m.vehiculo_capacidad,
            vehiculo_modelo: m.vehiculo_modelo,
            transporte_id: m.transporte_id,
            transporte_nombre: m.transporte_nombre,
            transporte_ruc: m.transporte_ruc,
            transporte_telefono: m.transporte_telefono,
            conductor_brevete: m.conductor_brevete,
            conductor_nombre: m.conductor_nombre,
            conductor_apellidos: m.conductor_apellidos,
            conductor_telefono: m.conductor_telefono,
        }
    }
}

/// Request para asignar vehículo a un file_tour específico
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignVehiculoToFileTourRequest {
    /// ID del file_tour al que se asigna el vehículo
    pub id_file_tour: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    #[validate(range(min = 0, message = "Capacidad asignada no puede ser negativa"))]
    pub capacidad_asignada: Option<i32>,
}

/// Request para actualizar un file_vehiculo (cambiar vehículo, conductor, capacidad o status)
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileVehiculoRequest {
    /// Nuevo vehículo asignado (cambia también capacidad_asignada si no se especifica)
    pub id_vehiculo: Option<i32>,
    /// Nuevo conductor asignado
    pub id_conductor: Option<i32>,
    /// Si true, quita el conductor (setea a NULL). Tiene prioridad sobre id_conductor.
    #[serde(default)]
    pub clear_conductor: bool,
    /// Nueva capacidad asignada
    #[validate(range(min = 1, message = "Capacidad asignada debe ser al menos 1"))]
    pub capacidad_asignada: Option<i32>,
    /// Nuevo status: reservado, confirmado, cancelado, etc.
    #[validate(length(min = 1, max = 20))]
    pub status: Option<String>,
}

/// Request para actualizar un file_guia (cambiar guía, rol, file_tour, status)
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileGuiaRequest {
    /// Nuevo guía asignado
    pub id_guia: Option<i32>,
    /// Nuevo rol del guía (e.g. "principal", "auxiliar")
    #[validate(length(min = 1, max = 50))]
    pub rol: Option<String>,
    /// Si true, limpia el rol (lo pone en NULL)
    #[serde(default)]
    pub clear_rol: bool,
    /// Nuevo file_tour (mover la asignación a otro tour)
    pub id_file_tour: Option<i32>,
    /// Nuevo status: asignado, confirmado, cancelado, etc.
    #[validate(length(min = 1, max = 20))]
    pub status: Option<String>,
}

// ==================== FILE DETALLE COMPLETO ====================

/// Respuesta para cambio de status de recursos asignados
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ResourceStatusUpdateResponse {
    pub resource_type: String, // "vehiculo", "guia", "conductor"
    pub resource_id: i32,
    pub old_status: String,
    pub new_status: String,
    pub message: String,
}

/// Request para cambio manual de status de vehículo
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateVehiculoStatusRequest {
    #[validate(length(min = 1, max = 20))]
    pub status: String, // "disponible", "ocupado", "en_servicio", "mantenimiento"
}

// Información de disponibilidad de vehículo
// ==================== MY FILES (Para usuarios autenticados) ====================

/// File asignado a un guía con todos los detalles necesarios
/// La relación principal es con file_tour (no directamente con file)
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsGuiaDto {
    // Info de la relación principal (file_tour)
    pub file_tour_id: i32,
    pub file_guia_id: i32,
    // Info del file (padre)
    pub file_id: i32,
    pub file_code: Option<String>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<String>,
    pub status: String,
    pub nro_pasajeros: i32,
    pub turno_tour: Option<String>,
    pub notas: Option<String>,
    // Info del tour (ampliada)
    pub tour_id: i32,
    pub tour_nombre: String,
    pub tour_lugar_inicio: String,
    pub tour_lugar_fin: String,
    pub tour_duracion_horas: Option<i32>,
    pub tour_tipo: Option<String>,
    // Info de la agencia
    pub agencia_id: i32,
    pub agencia_nombre: String,
    pub agencia_telefono: Option<String>,
    // Info del guía (este guía asignado)
    pub guia_id: i32,
    pub guia_nombre: String,
    pub guia_nro_carnet: String,
    pub rol_guia: Option<String>,
    pub asignado_at: DateTime<Utc>,
    // Estado de confirmación de la asignación
    pub estado_confirmacion: String, // "pendiente", "aceptado", "rechazado"
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<String>,
}

/// File asignado a un conductor/vehículo con todos los detalles
/// La relación principal es con file_tour (no directamente con file)
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsConductorDto {
    // Info de la relación principal (file_tour)
    pub file_tour_id: i32,
    pub file_vehiculo_id: i32,
    // Info del file (padre)
    pub file_id: i32,
    pub file_code: Option<String>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<String>,
    pub status: String,
    pub nro_pasajeros: i32,
    pub notas: Option<String>,
    // Info del tour
    pub tour_id: i32,
    pub tour_nombre: String,
    pub tour_lugar_inicio: String,
    pub tour_lugar_fin: String,
    // Info de la agencia
    pub agencia_id: i32,
    pub agencia_nombre: String,
    // Info del vehículo asignado
    pub vehiculo_id: i32,
    pub vehiculo_nombre: String,
    pub vehiculo_placa: String,
    pub vehiculo_capacidad: i32,
    pub asignado_at: DateTime<Utc>,
    // Estado de confirmación de la asignación
    pub estado_confirmacion: String, // "pendiente", "aceptado", "rechazado"
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<String>,
}

/// File asignado a un restaurante con todos los detalles
/// La relación principal es con file_tour (no directamente con file)
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsRestauranteDto {
    // Info de la relación principal (file_tour)
    pub file_tour_id: i32,
    pub file_restaurante_id: i32,
    // Info del file (padre)
    pub file_id: i32,
    pub file_code: Option<String>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub status: String,
    pub nro_pasajeros: i32,
    pub notas: Option<String>,
    // Info del tour
    pub tour_id: i32,
    pub tour_nombre: String,
    // Info de la agencia
    pub agencia_id: i32,
    pub agencia_nombre: String,
    // Info del servicio del restaurante
    pub tipo_servicio: Option<String>,
    pub dia: Option<i32>,
    pub asignado_at: DateTime<Utc>,
}

/// Response estándar para operaciones de confirmación
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ConfirmAssignmentResponse {
    pub success: bool,
    pub mensaje: String,
    pub estado_confirmacion: String,
    pub confirmado_at: Option<DateTime<Utc>>,
}

// ==================== UPDATE STATUS REQUESTS ====================

// Estados válidos para file relations
// - reservado: Estado inicial por defecto
// - pendiente: Solo para file_guias (guía no ha aceptado aún)
// - asignado: Recurso asignado y confirmado
// - en_curso: Servicio en progreso
// - completado: Servicio finalizado
// - cancelado: Asignación cancelada

/// Request para actualizar el status de una asignación
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateRelationStatusRequest {
    /// Estado: pendiente, reservado, asignado, confirmado, en_curso, completado, cancelado, anulado
    #[validate(length(
        min = 1,
        max = 20,
        message = "El status debe tener entre 1 y 20 caracteres"
    ))]
    pub status: String,
}

/// Response estándar para operaciones de actualización de status
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateStatusResponse {
    pub success: bool,
    pub mensaje: String,
    pub old_status: String,
    pub new_status: String,
}

/// Enum para validar estados permitidos en relaciones de file
/// Todos los estados son válidos para todas las relaciones
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRelationStatus {
    Pendiente,
    Reservado,
    Asignado,
    Confirmado,
    EnCurso,
    Completado,
    Cancelado,
    Anulado,
    NoShow,
    Pagado,
}

impl FileRelationStatus {
    /// Convierte string a enum, retorna error si no es válido
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "pendiente" => Ok(Self::Pendiente),
            "reservado" => Ok(Self::Reservado),
            "asignado" => Ok(Self::Asignado),
            "confirmado" => Ok(Self::Confirmado),
            "en_curso" => Ok(Self::EnCurso),
            "completado" => Ok(Self::Completado),
            "cancelado" => Ok(Self::Cancelado),
            "anulado" => Ok(Self::Anulado),
            "no_show" => Ok(Self::NoShow),
            "pagado" => Ok(Self::Pagado),
            _ => Err(format!("Status invalido: {}. Valores permitidos: pendiente, reservado, asignado, confirmado, en_curso, completado, cancelado, anulado, no_show, pagado", s))
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pendiente => "pendiente",
            Self::Reservado => "reservado",
            Self::Asignado => "asignado",
            Self::Confirmado => "confirmado",
            Self::EnCurso => "en_curso",
            Self::Completado => "completado",
            Self::Cancelado => "cancelado",
            Self::Anulado => "anulado",
            Self::NoShow => "no_show",
            Self::Pagado => "pagado",
        }
    }
}
