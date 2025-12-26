//! # Entrada Entity
//! 
//! Entidad para entradas/tickets a lugares turísticos.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipo de entrada
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

/// Entidad Entrada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrada {
    pub id: Uuid,
    pub nombre: String,
    pub precio: f64,
    pub ruta: Option<String>,  // Lugar/ruta del ticket
    pub tipo: TipoEntrada,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entrada {
    pub fn new(nombre: String, precio: f64, tipo: TipoEntrada) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            nombre,
            precio,
            ruta: None,
            tipo,
            descripcion: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Precio formateado
    pub fn precio_formateado(&self) -> String {
        format!("S/ {:.2}", self.precio)
    }
}
