
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HorarioRestaurante {
    pub apertura: Option<String>,
    pub cierre: Option<String>,
    pub dias: Vec<String>,  // ["Lunes", "Martes", ...]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restaurante {
    pub id: i32,
    pub nombre: String,
    pub direccion: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub tipo_atencion: Option<JsonValue>,  // ["desayuno", "almuerzo", "cena"]
    pub precio_promedio: Option<BigDecimal>,
    pub capacidad: Option<i32>,
    pub horario: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub encargado: Option<i32>,
}

impl Restaurante {
    pub fn new(nombre: String, direccion: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            direccion,
            telefono: None,
            correo: None,
            tipo_atencion: Some(serde_json::json!(["almuerzo"])),
            precio_promedio: None,
            capacidad: None,
            horario: Some(serde_json::json!({})),
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
            encargado: None,
        }
    }
    
    /// Obtiene los tipos de atención
    pub fn get_tipo_atencion(&self) -> Vec<String> {
        self.tipo_atencion.as_ref()
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default()
    }
    
    /// Obtiene el horario tipado
    pub fn get_horario(&self) -> Option<HorarioRestaurante> {
        self.horario.as_ref()
            .and_then(|h| serde_json::from_value(h.clone()).ok())
    }
    
    /// Verifica si atiende un tipo específico
    pub fn atiende(&self, tipo: &str) -> bool {
        self.get_tipo_atencion().iter()
            .any(|t| t.to_lowercase() == tipo.to_lowercase())
    }
}
