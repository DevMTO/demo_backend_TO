use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::{Vehiculo, StatusVehiculo};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct VehiculoResponse {
    pub id: i32,
    pub id_transporte: i32,
    pub nombre: String,
    pub modelo: Option<String>,
    pub placa: String,
    pub capacidad: i32,
    pub capacidad_disponible: i32,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Vehiculo> for VehiculoResponse {
    fn from(v: Vehiculo) -> Self {
        Self {
            id: v.id,
            id_transporte: v.id_transporte,
            nombre: v.nombre,
            modelo: v.modelo,
            placa: v.placa,
            capacidad: v.capacidad,
            capacidad_disponible: v.capacidad_disponible,
            status: v.status.to_string(), // Enum → String
            is_active: v.is_active,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

/// DTO para listar vehículos con el nombre del transporte
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct VehiculoListItemDto {
    pub id: i32,
    pub id_transporte: i32,
    pub transporte_nombre: Option<String>,
    pub nombre: String,
    pub modelo: Option<String>,
    pub placa: String,
    pub capacidad: i32,
    pub capacidad_disponible: i32,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateVehiculoRequest {
    pub id_transporte: i32,
    
    #[validate(length(min = 2, max = 100, message = "Nombre debe tener entre 2 y 100 caracteres"))]
    pub nombre: String,
    
    #[validate(length(max = 50))]
    pub modelo: Option<String>,
    
    #[validate(length(min = 6, max = 10, message = "Placa inválida"))]
    pub placa: String,
    
    #[validate(range(min = 1, max = 100, message = "Capacidad entre 1 y 100"))]
    pub capacidad: i32,
}

impl CreateVehiculoRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Vehiculo {
        let now = Utc::now();
        Vehiculo {
            id: 0,
            id_transporte: self.id_transporte,
            nombre: self.nombre,
            modelo: self.modelo,
            placa: self.placa.to_uppercase(),
            capacidad: self.capacidad,
            capacidad_disponible: self.capacidad,
            status: StatusVehiculo::Disponible,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateVehiculoRequest {
    pub id_transporte: Option<i32>,
    
    #[validate(length(min = 2, max = 100))]
    pub nombre: Option<String>,
    
    #[validate(length(max = 50))]
    pub modelo: Option<String>,
    
    #[validate(length(min = 6, max = 10))]
    pub placa: Option<String>,
    
    #[validate(range(min = 1, max = 100))]
    pub capacidad: Option<i32>,
    
    #[validate(length(max = 20))]
    pub status: Option<String>,
    
    pub is_active: Option<bool>,
}

impl UpdateVehiculoRequest {
    pub fn apply_to(self, mut vehiculo: Vehiculo, updated_by: Option<i32>) -> Vehiculo {
        if let Some(id_transporte) = self.id_transporte {
            vehiculo.id_transporte = id_transporte;
        }
        if let Some(nombre) = self.nombre {
            vehiculo.nombre = nombre;
        }
        if let Some(modelo) = self.modelo {
            vehiculo.modelo = Some(modelo);
        }
        if let Some(placa) = self.placa {
            vehiculo.placa = placa.to_uppercase();
        }
        if let Some(capacidad) = self.capacidad {
            vehiculo.capacidad = capacidad;
        }
        if let Some(status) = self.status {
            // Parse String to enum, keep old value if invalid
            if let Ok(status_enum) = status.parse::<StatusVehiculo>() {
                vehiculo.status = status_enum;
            }
        }
        if let Some(is_active) = self.is_active {
            vehiculo.is_active = is_active;
        }
        vehiculo.updated_by = updated_by;
        vehiculo.updated_at = Utc::now();
        vehiculo
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct VehiculoListResponse {
    pub items: Vec<VehiculoListItemDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
