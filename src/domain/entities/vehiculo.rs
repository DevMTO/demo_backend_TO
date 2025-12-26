//! # Vehiculo Entity
//! 
//! Entidad para vehículos de transporte.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Estado del vehículo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusVehiculo {
    Disponible,
    EnUso,
    Mantenimiento,
    FueraServicio,
}

impl std::fmt::Display for StatusVehiculo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusVehiculo::Disponible => write!(f, "disponible"),
            StatusVehiculo::EnUso => write!(f, "en_uso"),
            StatusVehiculo::Mantenimiento => write!(f, "mantenimiento"),
            StatusVehiculo::FueraServicio => write!(f, "fuera_servicio"),
        }
    }
}

impl std::str::FromStr for StatusVehiculo {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disponible" => Ok(StatusVehiculo::Disponible),
            "en_uso" => Ok(StatusVehiculo::EnUso),
            "mantenimiento" => Ok(StatusVehiculo::Mantenimiento),
            "fuera_servicio" => Ok(StatusVehiculo::FueraServicio),
            _ => Err(format!("Status de vehículo inválido: {s}")),
        }
    }
}

impl Default for StatusVehiculo {
    fn default() -> Self {
        StatusVehiculo::Disponible
    }
}

/// Entidad Vehículo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehiculo {
    pub id: Uuid,
    pub id_transporte: Uuid,
    pub nombre: String,
    pub modelo: Option<String>,
    pub placa: String,
    pub capacidad: i32,
    pub status: StatusVehiculo,
    pub media: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Vehiculo {
    pub fn new(id_transporte: Uuid, nombre: String, placa: String, capacidad: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            id_transporte,
            nombre,
            modelo: None,
            placa,
            capacidad,
            status: StatusVehiculo::Disponible,
            media: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Verifica si está disponible
    pub fn esta_disponible(&self) -> bool {
        self.status == StatusVehiculo::Disponible
    }
    
    /// Info resumida del vehículo
    pub fn info_resumida(&self) -> String {
        format!("{} - {} ({})", self.nombre, self.placa, self.capacidad)
    }
}
