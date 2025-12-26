//! # Persona Entity
//! 
//! Entidad base para datos personales (usado por usuarios, conductores, guías, etc.)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipos de documento de identidad
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoDocumento {
    Dni,
    Pasaporte,
    CarnetExtranjeria,
    Ruc,
    Otro,
}

impl std::fmt::Display for TipoDocumento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoDocumento::Dni => write!(f, "DNI"),
            TipoDocumento::Pasaporte => write!(f, "PASAPORTE"),
            TipoDocumento::CarnetExtranjeria => write!(f, "CARNET_EXTRANJERIA"),
            TipoDocumento::Ruc => write!(f, "RUC"),
            TipoDocumento::Otro => write!(f, "OTRO"),
        }
    }
}

impl std::str::FromStr for TipoDocumento {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DNI" => Ok(TipoDocumento::Dni),
            "PASAPORTE" => Ok(TipoDocumento::Pasaporte),
            "CARNET_EXTRANJERIA" => Ok(TipoDocumento::CarnetExtranjeria),
            "RUC" => Ok(TipoDocumento::Ruc),
            "OTRO" => Ok(TipoDocumento::Otro),
            _ => Err(format!("Tipo de documento inválido: {s}")),
        }
    }
}

impl Default for TipoDocumento {
    fn default() -> Self {
        TipoDocumento::Dni
    }
}

/// Entidad Persona
/// 
/// Datos personales básicos que pueden ser referenciados por usuarios,
/// conductores, guías, y otros roles del sistema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: Uuid,
    pub tipo_documento: TipoDocumento,
    pub nro_documento: String,
    pub nombre: String,
    pub apellidos: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Persona {
    /// Crear una nueva persona
    pub fn new(
        tipo_documento: TipoDocumento,
        nro_documento: String,
        nombre: String,
        apellidos: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tipo_documento,
            nro_documento,
            nombre,
            apellidos,
            telefono: None,
            correo: None,
            fecha_nacimiento: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Nombre completo
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombre, self.apellidos)
    }
    
    /// Documento formateado
    pub fn documento_formateado(&self) -> String {
        format!("{}: {}", self.tipo_documento, self.nro_documento)
    }
}
