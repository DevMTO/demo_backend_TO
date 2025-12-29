use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    /// Entidad no encontrada
    EntityNotFound { entity: String, id: String },
    
    /// Violación de regla de negocio
    BusinessRuleViolation(String),
    
    /// Email inválido
    InvalidEmail(String),
    
    /// Contraseña inválida
    InvalidPassword(String),
    
    /// Usuario inactivo
    InactiveUser,
    
    /// Email no verificado
    EmailNotVerified,
    
    /// MFA requerido
    MfaRequired,
    
    /// Código MFA inválido
    InvalidMfaCode,
    
    /// Sesión expirada
    SessionExpired,
    
    /// Token inválido
    InvalidToken,
    
    /// Documento duplicado
    DuplicateDocument { document_type: String, document_number: String },
    
    /// Datos inválidos
    InvalidData(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::EntityNotFound { entity, id } => {
                write!(f, "{} con ID '{}' no encontrado", entity, id)
            }
            DomainError::BusinessRuleViolation(msg) => {
                write!(f, "Violación de regla de negocio: {}", msg)
            }
            DomainError::InvalidEmail(msg) => write!(f, "Email inválido: {}", msg),
            DomainError::InvalidPassword(msg) => write!(f, "Contraseña inválida: {}", msg),
            DomainError::InactiveUser => write!(f, "Usuario inactivo"),
            DomainError::EmailNotVerified => write!(f, "Email no verificado"),
            DomainError::MfaRequired => write!(f, "Autenticación de dos factores requerida"),
            DomainError::InvalidMfaCode => write!(f, "Código MFA inválido"),
            DomainError::SessionExpired => write!(f, "Sesión expirada"),
            DomainError::InvalidToken => write!(f, "Token inválido"),
            DomainError::DuplicateDocument { document_type, document_number } => {
                write!(f, "Documento {} {} ya existe", document_type, document_number)
            }
            DomainError::InvalidData(msg) => write!(f, "Datos inválidos: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}
