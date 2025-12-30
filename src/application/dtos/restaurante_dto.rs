use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Restaurante;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RestauranteResponse {
    pub id: i32,
    pub nombre: String,
    pub direccion: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    #[ts(type = "object | null")]
    pub tipo_atencion: Option<JsonValue>,
    #[ts(type = "string | null")]
    pub precio_promedio: Option<BigDecimal>,
    pub capacidad: Option<i32>,
    #[ts(type = "object | null")]
    pub horario: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Restaurante> for RestauranteResponse {
    fn from(r: Restaurante) -> Self {
        Self {
            id: r.id,
            nombre: r.nombre,
            direccion: r.direccion,
            telefono: r.telefono,
            correo: r.correo,
            tipo_atencion: r.tipo_atencion,
            precio_promedio: r.precio_promedio,
            capacidad: r.capacidad,
            horario: r.horario,
            encargado: r.encargado,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// DTO para listado con nombre del encargado
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RestauranteListItemDto {
    pub id: i32,
    pub nombre: String,
    pub direccion: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    #[ts(type = "object | null")]
    pub tipo_atencion: Option<JsonValue>,
    #[ts(type = "string | null")]
    pub precio_promedio: Option<BigDecimal>,
    pub capacidad: Option<i32>,
    #[ts(type = "object | null")]
    pub horario: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub encargado_nombre: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateRestauranteRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    #[validate(length(min = 5, message = "Dirección muy corta"))]
    pub direccion: String,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    /// Array de tipos: ["desayuno", "almuerzo", "cena", "box lunch"]
    pub tipo_atencion: Option<Vec<String>>,
    
    #[validate(range(min = 0.0, message = "Precio debe ser positivo"))]
    pub precio_promedio: Option<f64>,
    
    #[validate(range(min = 1, message = "Capacidad mínima 1"))]
    pub capacidad: Option<i32>,
    
    /// {"lunes": "8:00-22:00", "martes": "8:00-22:00", ...}
    #[ts(type = "object | null")]
    pub horario: Option<JsonValue>,
    
    pub encargado: Option<i32>,
}

impl CreateRestauranteRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Restaurante {
        let now = Utc::now();
        Restaurante {
            id: 0,
            nombre: self.nombre,
            direccion: self.direccion,
            telefono: self.telefono,
            correo: self.correo,
            tipo_atencion: self.tipo_atencion.map(|t| serde_json::json!(t)),
            precio_promedio: self.precio_promedio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
            capacidad: self.capacidad,
            horario: self.horario,
            encargado: self.encargado,
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
pub struct UpdateRestauranteRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    #[validate(length(min = 5))]
    pub direccion: Option<String>,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    pub tipo_atencion: Option<Vec<String>>,
    
    #[validate(range(min = 0.0))]
    pub precio_promedio: Option<f64>,
    
    #[validate(range(min = 1))]
    pub capacidad: Option<i32>,
    
    #[ts(type = "object | null")]
    pub horario: Option<JsonValue>,
    
    pub encargado: Option<i32>,
    
    pub is_active: Option<bool>,
}

impl UpdateRestauranteRequest {
    pub fn apply_to(self, mut restaurante: Restaurante, updated_by: Option<i32>) -> Restaurante {
        if let Some(nombre) = self.nombre {
            restaurante.nombre = nombre;
        }
        if let Some(direccion) = self.direccion {
            restaurante.direccion = direccion;
        }
        if let Some(telefono) = self.telefono {
            restaurante.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            restaurante.correo = Some(correo);
        }
        if let Some(tipo_atencion) = self.tipo_atencion {
            restaurante.tipo_atencion = Some(serde_json::json!(tipo_atencion));
        }
        if let Some(precio) = self.precio_promedio {
            restaurante.precio_promedio = Some(BigDecimal::try_from(precio).unwrap_or_default());
        }
        if let Some(capacidad) = self.capacidad {
            restaurante.capacidad = Some(capacidad);
        }
        if let Some(horario) = self.horario {
            restaurante.horario = Some(horario);
        }
        if let Some(encargado) = self.encargado {
            restaurante.encargado = Some(encargado);
        }
        if let Some(is_active) = self.is_active {
            restaurante.is_active = is_active;
        }
        restaurante.updated_by = updated_by;
        restaurante.updated_at = Utc::now();
        restaurante
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestauranteListResponse {
    pub items: Vec<RestauranteResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
