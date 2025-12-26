//! # Document Type Entity
//! 
//! Tipos de documento de identidad (DNI, Pasaporte, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tipos de documento de identidad predefinidos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentTypeCode {
    /// Documento Nacional de Identidad
    DNI,
    /// Pasaporte
    Passport,
    /// Carné de Extranjería
    ForeignerCard,
    /// RUC (para empresas)
    RUC,
    /// Otro tipo de documento
    Other(String),
}

impl std::fmt::Display for DocumentTypeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentTypeCode::DNI => write!(f, "DNI"),
            DocumentTypeCode::Passport => write!(f, "PASSPORT"),
            DocumentTypeCode::ForeignerCard => write!(f, "FOREIGNER_CARD"),
            DocumentTypeCode::RUC => write!(f, "RUC"),
            DocumentTypeCode::Other(code) => write!(f, "{}", code),
        }
    }
}

impl std::str::FromStr for DocumentTypeCode {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DNI" => Ok(DocumentTypeCode::DNI),
            "PASSPORT" => Ok(DocumentTypeCode::Passport),
            "FOREIGNER_CARD" => Ok(DocumentTypeCode::ForeignerCard),
            "RUC" => Ok(DocumentTypeCode::RUC),
            other => Ok(DocumentTypeCode::Other(other.to_string())),
        }
    }
}

/// Entidad de Tipo de Documento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentType {
    /// ID del tipo de documento
    pub id: i32,
    /// Código único del tipo
    pub code: String,
    /// Nombre descriptivo
    pub name: String,
    /// Regex para validar el formato (opcional)
    pub format_regex: Option<String>,
    /// Si está activo
    pub is_active: bool,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de actualización
    pub updated_at: DateTime<Utc>,
}

impl DocumentType {
    /// Crear un nuevo tipo de documento
    pub fn new(id: i32, code: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            code,
            name,
            format_regex: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Crear con regex de validación
    pub fn with_format_regex(mut self, regex: String) -> Self {
        self.format_regex = Some(regex);
        self
    }
    
    /// Validar número de documento según el formato
    pub fn validate_document_number(&self, number: &str) -> bool {
        match &self.format_regex {
            Some(regex_str) => {
                if let Ok(regex) = regex::Regex::new(regex_str) {
                    regex.is_match(number)
                } else {
                    true // Si el regex es inválido, permitir
                }
            }
            None => true,
        }
    }
}

/// Tipos de documento predefinidos
impl DocumentType {
    /// DNI peruano (8 dígitos)
    pub fn dni() -> Self {
        Self::new(1, "DNI".to_string(), "Documento Nacional de Identidad".to_string())
            .with_format_regex(r"^\d{8}$".to_string())
    }
    
    /// Pasaporte
    pub fn passport() -> Self {
        Self::new(2, "PASSPORT".to_string(), "Pasaporte".to_string())
            .with_format_regex(r"^[A-Z0-9]{6,12}$".to_string())
    }
    
    /// Carné de extranjería
    pub fn foreigner_card() -> Self {
        Self::new(3, "FOREIGNER_CARD".to_string(), "Carné de Extranjería".to_string())
            .with_format_regex(r"^[A-Z0-9]{9,12}$".to_string())
    }
    
    /// RUC (11 dígitos)
    pub fn ruc() -> Self {
        Self::new(4, "RUC".to_string(), "Registro Único de Contribuyentes".to_string())
            .with_format_regex(r"^\d{11}$".to_string())
    }
}
