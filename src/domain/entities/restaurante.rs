//! # Restaurante Entity
//! 
//! Entidad para restaurantes.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Horario del restaurante
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HorarioRestaurante {
    pub apertura: Option<String>,
    pub cierre: Option<String>,
    pub dias: Vec<String>,  // ["Lunes", "Martes", ...]
}

/// Entidad Restaurante
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restaurante {
    pub id: Uuid,
    pub nombre: String,
    pub direccion: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub tipo_atencion: JsonValue,  // ["desayuno", "almuerzo", "cena"]
    pub precio_promedio: f64,
    pub capacidad: Option<i32>,
    pub horario: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Restaurante {
    pub fn new(nombre: String, direccion: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            nombre,
            direccion,
            telefono: None,
            correo: None,
            tipo_atencion: serde_json::json!(["almuerzo"]),
            precio_promedio: 0.0,
            capacidad: None,
            horario: serde_json::json!({}),
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene los tipos de atención
    pub fn get_tipo_atencion(&self) -> Vec<String> {
        serde_json::from_value(self.tipo_atencion.clone()).unwrap_or_default()
    }
    
    /// Obtiene el horario tipado
    pub fn get_horario(&self) -> Option<HorarioRestaurante> {
        serde_json::from_value(self.horario.clone()).ok()
    }
    
    /// Verifica si atiende un tipo específico
    pub fn atiende(&self, tipo: &str) -> bool {
        self.get_tipo_atencion().iter()
            .any(|t| t.to_lowercase() == tipo.to_lowercase())
    }
}
