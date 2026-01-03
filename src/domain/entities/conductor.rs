
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusConductor {
    Disponible,
    EnServicio,
    Inactivo,
    Suspendido,
}

impl std::fmt::Display for StatusConductor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusConductor::Disponible => write!(f, "disponible"),
            StatusConductor::EnServicio => write!(f, "en_servicio"),
            StatusConductor::Inactivo => write!(f, "inactivo"),
            StatusConductor::Suspendido => write!(f, "suspendido"),
        }
    }
}

impl std::str::FromStr for StatusConductor {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disponible" => Ok(StatusConductor::Disponible),
            "en_servicio" => Ok(StatusConductor::EnServicio),
            "inactivo" => Ok(StatusConductor::Inactivo),
            "suspendido" => Ok(StatusConductor::Suspendido),
            _ => Err(format!("Status de conductor inválido: {s}")),
        }
    }
}

impl Default for StatusConductor {
    fn default() -> Self {
        StatusConductor::Disponible
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conductor {
    pub id: i32,
    pub id_persona: i32,
    pub id_transporte: Option<i32>,
    pub nro_brevete: String,
    pub tiene_soat: bool,
    pub status: StatusConductor,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Conductor {
    pub fn new(id_persona: i32, nro_brevete: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_persona,
            id_transporte: None,
            nro_brevete,
            tiene_soat: false,
            status: StatusConductor::Disponible,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Verifica si está disponible
    pub fn esta_disponible(&self) -> bool {
        self.status == StatusConductor::Disponible
    }
    
    /// Verifica si tiene documentación completa
    pub fn documentacion_completa(&self) -> bool {
        !self.nro_brevete.is_empty() && self.tiene_soat
    }
}
