use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::infrastructure::persistence::models::{
    FileEntradaModel, FileGuiaModel, FilePasajeroModel, 
    FileRestauranteModel, FileVehiculoModel
};

// ==================== FILE ENTRADA ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileEntradaResponse {
    pub id: i32,
    pub id_file: i32,
    pub id_entrada: i32,
    pub cantidad: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // Datos de la entrada relacionada (se pueden poblar en el handler)
    pub entrada_nombre: Option<String>,
    pub entrada_precio: Option<String>,
}

impl From<FileEntradaModel> for FileEntradaResponse {
    fn from(m: FileEntradaModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_entrada: m.id_entrada,
            cantidad: m.cantidad,
            created_at: m.created_at,
            created_by: m.created_by,
            entrada_nombre: None,
            entrada_precio: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignEntradaToFileRequest {
    pub id_entrada: i32,
    #[validate(range(min = 1, message = "La cantidad debe ser al menos 1"))]
    pub cantidad: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct BulkAssignEntradasRequest {
    pub entradas: Vec<AssignEntradaToFileRequest>,
}

// ==================== FILE GUIA ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileGuiaResponse {
    pub id: i32,
    pub id_file: i32,
    pub id_guia: i32,
    pub rol: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // Datos del guía relacionado
    pub guia_nombre: Option<String>,
    pub guia_nro_carnet: Option<String>,
}

impl From<FileGuiaModel> for FileGuiaResponse {
    fn from(m: FileGuiaModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_guia: m.id_guia,
            rol: m.rol,
            created_at: m.created_at,
            created_by: m.created_by,
            guia_nombre: None,
            guia_nro_carnet: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignGuiaToFileRequest {
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
    pub id_persona: i32,
    pub asiento: Option<String>,
    pub tipo_pasajero: Option<String>,
    pub notas: Option<String>,
    pub nacionalidad: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // Datos del pasajero relacionado
    pub pasajero_nombre: Option<String>,
    pub pasajero_apellidos: Option<String>,
    pub pasajero_documento: Option<String>,
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
            created_at: m.created_at,
            created_by: m.created_by,
            pasajero_nombre: None,
            pasajero_apellidos: None,
            pasajero_documento: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AddPasajeroToFileRequest {
    pub id_persona: i32,
    #[validate(length(max = 10))]
    pub asiento: Option<String>,
    #[validate(length(max = 30))]
    pub tipo_pasajero: Option<String>, // "adulto", "niño", "infante"
    #[validate(length(max = 60))]
    pub nacionalidad: Option<String>,
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
    
    #[validate(length(min = 6, max = 20, message = "Nro documento debe tener entre 6 y 20 caracteres"))]
    pub nro_documento: String,
    
    #[validate(length(min = 2, max = 100, message = "Nombre debe tener entre 2 y 100 caracteres"))]
    pub nombre: String,
    
    #[validate(length(min = 2, max = 100, message = "Apellidos debe tener entre 2 y 100 caracteres"))]
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

// ==================== FILE RESTAURANTE ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileRestauranteResponse {
    pub id: i32,
    pub id_file: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<String>,
    pub dia: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // Datos del restaurante relacionado
    pub restaurante_nombre: Option<String>,
    pub restaurante_direccion: Option<String>,
}

impl From<FileRestauranteModel> for FileRestauranteResponse {
    fn from(m: FileRestauranteModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_restaurante: m.id_restaurante,
            tipo_servicio: m.tipo_servicio,
            dia: m.dia,
            created_at: m.created_at,
            created_by: m.created_by,
            restaurante_nombre: None,
            restaurante_direccion: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignRestauranteToFileRequest {
    pub id_restaurante: i32,
    #[validate(length(max = 30))]
    pub tipo_servicio: Option<String>, // "desayuno", "almuerzo", "cena"
    #[validate(range(min = 1))]
    pub dia: Option<i32>, // Día del tour (1, 2, 3, etc.)
}

// ==================== FILE VEHICULO ====================

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileVehiculoResponse {
    pub id: i32,
    pub id_file: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // Datos del vehículo relacionado
    pub vehiculo_nombre: Option<String>,
    pub vehiculo_placa: Option<String>,
    pub vehiculo_capacidad: Option<i32>,
    // Datos del conductor relacionado
    pub conductor_nombre: Option<String>,
    pub conductor_brevete: Option<String>,
}

impl From<FileVehiculoModel> for FileVehiculoResponse {
    fn from(m: FileVehiculoModel) -> Self {
        Self {
            id: m.id,
            id_file: m.id_file,
            id_vehiculo: m.id_vehiculo,
            id_conductor: m.id_conductor,
            created_at: m.created_at,
            created_by: m.created_by,
            vehiculo_nombre: None,
            vehiculo_placa: None,
            vehiculo_capacidad: None,
            conductor_nombre: None,
            conductor_brevete: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AssignVehiculoToFileRequest {
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
}

// ==================== FILE DETALLE COMPLETO ====================

/// Respuesta completa de un File con todos sus datos relacionados
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileDetailResponse {
    // Datos básicos del file
    pub id: i32,
    pub id_tour: i32,
    pub id_agencia: i32,
    pub fecha_inicio: String, // NaiveDate como string
    pub fecha_fin: String,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<String>, // NaiveTime como string
    pub notas: Option<String>,
    pub status: String,
    #[ts(type = "string")]
    pub monto_total: String,
    #[ts(type = "string")]
    pub monto_pagado: String,
    #[ts(type = "string")]
    pub saldo_pendiente: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Datos del tour relacionado
    pub tour_nombre: Option<String>,
    pub tour_lugar_inicio: Option<String>,
    pub tour_lugar_fin: Option<String>,
    
    // Datos de la agencia relacionada
    pub agencia_nombre: Option<String>,
    
    // Totales de asignaciones
    pub total_pasajeros: i32,
    pub total_entradas: i32,
    pub total_guias: i32,
    pub total_vehiculos: i32,
    pub total_restaurantes: i32,
    
    // Listas de asignaciones
    pub entradas: Vec<FileEntradaResponse>,
    pub guias: Vec<FileGuiaResponse>,
    pub pasajeros: Vec<FilePasajeroResponse>,
    pub restaurantes: Vec<FileRestauranteResponse>,
    pub vehiculos: Vec<FileVehiculoResponse>,
}

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

/// Información de disponibilidad de vehículo
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct VehiculoDisponibilidadResponse {
    pub id: i32,
    pub nombre: String,
    pub placa: String,
    pub capacidad: i32,
    pub status: String,
    pub pax_asignados: i32,
    pub pax_disponibles: i32,
    pub files_asignados: Vec<i32>, // IDs de files donde está asignado
    pub puede_asignar_mas: bool,
}

// ==================== MY FILES (Para usuarios autenticados) ====================

/// File asignado a un guía con todos los detalles necesarios
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsGuiaDto {
    // Info del file
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
}

/// File asignado a un conductor/vehículo con todos los detalles
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsConductorDto {
    // Info del file
    pub file_id: i32,
    pub file_code: Option<String>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<String>,
    pub status: String,
    pub nro_pasajeros: i32,
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
}

/// File asignado a un restaurante con todos los detalles
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct MyFileAsRestauranteDto {
    // Info del file
    pub file_id: i32,
    pub file_code: Option<String>,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub status: String,
    pub nro_pasajeros: i32,
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
