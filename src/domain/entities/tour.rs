use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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

/// Estructura para un rango horario (inicio y fin)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangoHorario {
    pub start: String,  // "HH:MM"
    pub end: String,    // "HH:MM"
}

/// Estructura para los horarios del tour
/// Puede tener: "full" (día completo), "morning" (mañana), "afternoon" (tarde)
/// O combinaciones: "morning" + "afternoon" para tours half day con ambos turnos
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HorariosTour {
    pub full: Option<RangoHorario>,
    pub morning: Option<RangoHorario>,
    pub afternoon: Option<RangoHorario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tour {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: Option<String>,
    pub lugar_fin: Option<String>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub duracion_dias: Option<i32>,
    pub media: Option<JsonValue>,
    pub tipo_tour: Option<String>,
    pub horarios: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub tiene_restaurante: bool,
    pub geo_inicio: Option<JsonValue>,
    pub geo_fin: Option<JsonValue>,
    pub geo_ruta: Option<JsonValue>,
}

impl Tour {
    pub fn new(nombre: String, lugar_inicio: Option<String>, lugar_fin: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            lugar_inicio,
            lugar_fin,
            detalles: Some(serde_json::json!({})),
            itinerario: Some(serde_json::json!([])),
            duracion_dias: Some(1),
            media: Some(serde_json::json!({})),
            tipo_tour: Some(String::new()),
            horarios: Some(serde_json::json!({})),
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
            tiene_restaurante: false,
            geo_inicio: None,
            geo_fin: None,
            geo_ruta: None,
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
    
    /// Obtiene los horarios tipados
    pub fn get_horarios(&self) -> HorariosTour {
        self.horarios.as_ref()
            .and_then(|h| serde_json::from_value(h.clone()).ok())
            .unwrap_or_default()
    }
    
    /// Ruta formateada
    pub fn ruta(&self) -> String {
        format!("{} → {}", self.lugar_inicio.as_deref().unwrap_or("N/A"), self.lugar_fin.as_deref().unwrap_or("N/A"))
    }
}
