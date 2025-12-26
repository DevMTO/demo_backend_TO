//! # Tour Entity
//! 
//! Entidad para tours/paquetes turísticos.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Actividad del itinerario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActividadItinerario {
    pub dia: i32,
    pub hora: Option<String>,
    pub lugar: String,
    pub descripcion: String,
    pub duracion_minutos: Option<i32>,
}

/// Detalles del tour
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetallesTour {
    pub incluye: Vec<String>,
    pub no_incluye: Vec<String>,
    pub recomendaciones: Vec<String>,
    pub requisitos: Vec<String>,
}

/// Entidad Tour
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tour {
    pub id: Uuid,
    pub id_agencia: Uuid,
    pub nombre: String,
    pub lugar_inicio: String,
    pub lugar_fin: String,
    pub hora_inicio: Option<DateTime<Utc>>,
    pub hora_fin: Option<DateTime<Utc>>,
    pub detalles: JsonValue,
    pub itinerario: JsonValue,
    pub precio: f64,
    pub duracion_dias: i32,
    pub max_personas: Option<i32>,
    pub media: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tour {
    pub fn new(id_agencia: Uuid, nombre: String, lugar_inicio: String, lugar_fin: String, precio: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            id_agencia,
            nombre,
            lugar_inicio,
            lugar_fin,
            hora_inicio: None,
            hora_fin: None,
            detalles: serde_json::json!({}),
            itinerario: serde_json::json!([]),
            precio,
            duracion_dias: 1,
            max_personas: None,
            media: serde_json::json!({}),
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene el itinerario tipado
    pub fn get_itinerario(&self) -> Vec<ActividadItinerario> {
        serde_json::from_value(self.itinerario.clone()).unwrap_or_default()
    }
    
    /// Obtiene los detalles tipados
    pub fn get_detalles(&self) -> DetallesTour {
        serde_json::from_value(self.detalles.clone()).unwrap_or_default()
    }
    
    /// Ruta formateada
    pub fn ruta(&self) -> String {
        format!("{} → {}", self.lugar_inicio, self.lugar_fin)
    }
}
