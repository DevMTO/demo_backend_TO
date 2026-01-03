use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehiculo {
    pub id: i32,
    pub id_transporte: i32,
    pub nombre: String,
    pub modelo: Option<String>,
    pub placa: String,
    pub capacidad: i32,
    pub status: StatusVehiculo,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Vehiculo {
    pub fn new(id_transporte: i32, nombre: String, placa: String, capacidad: i32) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_transporte,
            nombre,
            modelo: None,
            placa,
            capacidad,
            status: StatusVehiculo::Disponible,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
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
