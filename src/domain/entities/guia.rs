//! # Guia Entity
//! 
//! Entidad para guías turísticos.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Estado del guía
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusGuia {
    Disponible,
    EnServicio,
    Inactivo,
    Suspendido,
}

impl std::fmt::Display for StatusGuia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusGuia::Disponible => write!(f, "disponible"),
            StatusGuia::EnServicio => write!(f, "en_servicio"),
            StatusGuia::Inactivo => write!(f, "inactivo"),
            StatusGuia::Suspendido => write!(f, "suspendido"),
        }
    }
}

impl std::str::FromStr for StatusGuia {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disponible" => Ok(StatusGuia::Disponible),
            "en_servicio" => Ok(StatusGuia::EnServicio),
            "inactivo" => Ok(StatusGuia::Inactivo),
            "suspendido" => Ok(StatusGuia::Suspendido),
            _ => Err(format!("Status de guía inválido: {s}")),
        }
    }
}

impl Default for StatusGuia {
    fn default() -> Self {
        StatusGuia::Disponible
    }
}

/// Entidad Guía
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guia {
    pub id: Uuid,
    pub id_persona: Uuid,
    pub nro_carnet: String,
    pub idiomas: JsonValue,        // ["Español", "Inglés"]
    pub especialidades: JsonValue, // ["City tours", "Aventura"]
    pub status: StatusGuia,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guia {
    pub fn new(id_persona: Uuid, nro_carnet: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            id_persona,
            nro_carnet,
            idiomas: serde_json::json!(["Español"]),
            especialidades: serde_json::json!([]),
            status: StatusGuia::Disponible,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene la lista de idiomas
    pub fn get_idiomas(&self) -> Vec<String> {
        serde_json::from_value(self.idiomas.clone()).unwrap_or_default()
    }
    
    /// Obtiene las especialidades
    pub fn get_especialidades(&self) -> Vec<String> {
        serde_json::from_value(self.especialidades.clone()).unwrap_or_default()
    }
    
    /// Verifica si está disponible
    pub fn esta_disponible(&self) -> bool {
        self.status == StatusGuia::Disponible
    }
    
    /// Verifica si habla un idioma específico
    pub fn habla_idioma(&self, idioma: &str) -> bool {
        self.get_idiomas().iter()
            .any(|i| i.to_lowercase() == idioma.to_lowercase())
    }
}
