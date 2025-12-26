//! # File Entity
//! 
//! Entidad para files de viaje (reservación de grupo/paquete).
//! Un "file" es una reservación completa que incluye tour, guías, 
//! pasajeros, vehículos, restaurantes y entradas.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Estado del file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusFile {
    Pendiente,
    Confirmado,
    EnCurso,
    Completado,
    Cancelado,
}

impl std::fmt::Display for StatusFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusFile::Pendiente => write!(f, "pendiente"),
            StatusFile::Confirmado => write!(f, "confirmado"),
            StatusFile::EnCurso => write!(f, "en_curso"),
            StatusFile::Completado => write!(f, "completado"),
            StatusFile::Cancelado => write!(f, "cancelado"),
        }
    }
}

impl std::str::FromStr for StatusFile {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pendiente" => Ok(StatusFile::Pendiente),
            "confirmado" => Ok(StatusFile::Confirmado),
            "en_curso" => Ok(StatusFile::EnCurso),
            "completado" => Ok(StatusFile::Completado),
            "cancelado" => Ok(StatusFile::Cancelado),
            _ => Err(format!("Status de file inválido: {s}")),
        }
    }
}

impl Default for StatusFile {
    fn default() -> Self {
        StatusFile::Pendiente
    }
}

/// Asignación de guía en el file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiaAsignado {
    pub id: Uuid,
    pub nombre: String,
    pub rol: String,  // "principal", "asistente"
}

/// Pasajero en el file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasajeroFile {
    pub id: Uuid,
    pub nombre: String,
    pub documento: String,
    pub asiento: Option<String>,
}

/// Vehículo asignado al file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiculoAsignado {
    pub id: Uuid,
    pub placa: String,
    pub conductor_id: Option<Uuid>,
    pub conductor_nombre: Option<String>,
}

/// Restaurante del file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestauranteFile {
    pub id: Uuid,
    pub nombre: String,
    pub tipo_servicio: String,
}

/// Entradas del file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaFile {
    pub id: Uuid,
    pub nombre: String,
    pub cantidad: i32,
    pub tipo: String,
    pub precio_unitario: f64,
}

/// Fechas del file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FechasFile {
    pub inicio: String,
    pub fin: String,
    pub dias: Vec<String>,
}

/// Entidad File (Reservación de grupo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub file_code: String,
    pub id_tour: Uuid,
    pub id_agencia: Uuid,
    
    // Asignaciones (JSONB)
    pub guias: JsonValue,
    pub pasajeros: JsonValue,
    pub vehiculos: JsonValue,
    pub restaurante: JsonValue,
    pub entradas: JsonValue,
    
    // Fechas y logística
    pub fechas: JsonValue,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<DateTime<Utc>>,
    pub notas: Option<String>,
    
    // Estado y financiero
    pub status: StatusFile,
    pub monto_total: f64,
    pub monto_pagado: f64,
    
    // Auditoría
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl File {
    pub fn new(id_tour: Uuid, id_agencia: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            file_code: String::new(),  // Se genera automáticamente en la BD
            id_tour,
            id_agencia,
            guias: serde_json::json!([]),
            pasajeros: serde_json::json!([]),
            vehiculos: serde_json::json!([]),
            restaurante: serde_json::json!({}),
            entradas: serde_json::json!([]),
            fechas: serde_json::json!({}),
            lugar_recojo: None,
            hora_recojo: None,
            notas: None,
            status: StatusFile::Pendiente,
            monto_total: 0.0,
            monto_pagado: 0.0,
            created_by: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene los guías asignados
    pub fn get_guias(&self) -> Vec<GuiaAsignado> {
        serde_json::from_value(self.guias.clone()).unwrap_or_default()
    }
    
    /// Obtiene los pasajeros
    pub fn get_pasajeros(&self) -> Vec<PasajeroFile> {
        serde_json::from_value(self.pasajeros.clone()).unwrap_or_default()
    }
    
    /// Obtiene los vehículos asignados
    pub fn get_vehiculos(&self) -> Vec<VehiculoAsignado> {
        serde_json::from_value(self.vehiculos.clone()).unwrap_or_default()
    }
    
    /// Obtiene las entradas
    pub fn get_entradas(&self) -> Vec<EntradaFile> {
        serde_json::from_value(self.entradas.clone()).unwrap_or_default()
    }
    
    /// Obtiene las fechas
    pub fn get_fechas(&self) -> Option<FechasFile> {
        serde_json::from_value(self.fechas.clone()).ok()
    }
    
    /// Número de pasajeros
    pub fn num_pasajeros(&self) -> usize {
        self.get_pasajeros().len()
    }
    
    /// Saldo pendiente
    pub fn saldo_pendiente(&self) -> f64 {
        self.monto_total - self.monto_pagado
    }
    
    /// Porcentaje pagado
    pub fn porcentaje_pagado(&self) -> f64 {
        if self.monto_total <= 0.0 {
            return 0.0;
        }
        (self.monto_pagado / self.monto_total) * 100.0
    }
    
    /// Verifica si está pagado completamente
    pub fn esta_pagado(&self) -> bool {
        self.monto_pagado >= self.monto_total
    }
    
    /// Verifica si se puede cancelar
    pub fn puede_cancelar(&self) -> bool {
        matches!(self.status, StatusFile::Pendiente | StatusFile::Confirmado)
    }
}
