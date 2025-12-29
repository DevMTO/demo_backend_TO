
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoEntrada {
    General,
    TuristaNacional,
    TuristaExtranjero,
    Estudiante,
    Menor,
    AdultoMayor,
}

impl std::fmt::Display for TipoEntrada {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoEntrada::General => write!(f, "general"),
            TipoEntrada::TuristaNacional => write!(f, "turista_nacional"),
            TipoEntrada::TuristaExtranjero => write!(f, "turista_extranjero"),
            TipoEntrada::Estudiante => write!(f, "estudiante"),
            TipoEntrada::Menor => write!(f, "menor"),
            TipoEntrada::AdultoMayor => write!(f, "adulto_mayor"),
        }
    }
}

impl std::str::FromStr for TipoEntrada {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "general" => Ok(TipoEntrada::General),
            "turista_nacional" => Ok(TipoEntrada::TuristaNacional),
            "turista_extranjero" => Ok(TipoEntrada::TuristaExtranjero),
            "estudiante" => Ok(TipoEntrada::Estudiante),
            "menor" => Ok(TipoEntrada::Menor),
            "adulto_mayor" => Ok(TipoEntrada::AdultoMayor),
            _ => Err(format!("Tipo de entrada inválido: {s}")),
        }
    }
}

impl Default for TipoEntrada {
    fn default() -> Self {
        TipoEntrada::General
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrada {
    pub id: i32,
    pub nombre: String,
    pub precio: BigDecimal,
    pub ruta: Option<String>,  // Lugar/ruta del ticket
    pub tipo: String, // Stored as varchar in DB
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Entrada {
    pub fn new(nombre: String, precio: BigDecimal, tipo: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            precio,
            ruta: None,
            tipo,
            descripcion: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
}
