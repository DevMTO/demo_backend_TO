use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActividadItinerario {
    pub dia: i32,
    pub hora: Option<String>,
    pub lugar: String,
    pub descripcion: String,
    pub duracion_minutos: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetallesTour {
    pub incluye: Vec<String>,
    pub no_incluye: Vec<String>,
    pub recomendaciones: Vec<String>,
    pub requisitos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tour {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: String,
    pub lugar_fin: String,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub media: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Tour {
    pub fn new(nombre: String, lugar_inicio: String, lugar_fin: String, precio_base: BigDecimal) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            lugar_inicio,
            lugar_fin,
            hora_inicio: None,
            hora_fin: None,
            detalles: Some(serde_json::json!({})),
            itinerario: Some(serde_json::json!([])),
            precio_base,
            duracion_dias: Some(1),
            media: Some(serde_json::json!({})),
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Obtiene el itinerario tipado
    pub fn get_itinerario(&self) -> Vec<ActividadItinerario> {
        self.itinerario.as_ref()
            .and_then(|i| serde_json::from_value(i.clone()).ok())
            .unwrap_or_default()
    }
    
    /// Obtiene los detalles tipados
    pub fn get_detalles(&self) -> DetallesTour {
        self.detalles.as_ref()
            .and_then(|d| serde_json::from_value(d.clone()).ok())
            .unwrap_or_default()
    }
    
    /// Ruta formateada
    pub fn ruta(&self) -> String {
        format!("{} → {}", self.lugar_inicio, self.lugar_fin)
    }
}
