//! # Email Value Object
//! 
//! Objeto de valor para emails validados.


use serde::{Deserialize, Serialize};
use std::fmt;

/// Email validado
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Crear un nuevo email validado
    pub fn new(email: impl Into<String>) -> Result<Self, EmailError> {
        let email = email.into().trim().to_lowercase();
        
        if email.is_empty() {
            return Err(EmailError::Empty);
        }
        
        if !Self::is_valid_format(&email) {
            return Err(EmailError::InvalidFormat);
        }
        
        Ok(Self(email))
    }
    
    /// Validar formato de email
    fn is_valid_format(email: &str) -> bool {
        // Validación básica de email
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let local = parts[0];
        let domain = parts[1];
        
        if local.is_empty() || domain.is_empty() {
            return false;
        }
        
        if !domain.contains('.') {
            return false;
        }
        
        // Verificar que no tenga caracteres inválidos
        let valid_chars = |c: char| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+';
        
        local.chars().all(valid_chars) && domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    }
    
    /// Obtener el valor del email
    pub fn value(&self) -> &str {
        &self.0
    }
    
    /// Obtener el dominio del email
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
    
    /// Obtener la parte local del email
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Errores de validación de email
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailError {
    Empty,
    InvalidFormat,
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::Empty => write!(f, "Email cannot be empty"),
            EmailError::InvalidFormat => write!(f, "Invalid email format"),
        }
    }
}

impl std::error::Error for EmailError {}