//! # Password Value Object
//! 
//! Objeto de valor para contraseñas con validación de fortaleza.


use serde::{Deserialize, Serialize};
use std::fmt;

/// Contraseña en texto plano (antes de hashear)
/// Solo se usa durante el registro/login, nunca se persiste
#[derive(Clone)]
pub struct PlainPassword(String);

impl PlainPassword {
    /// Crear una nueva contraseña validada
    pub fn new(password: impl Into<String>) -> Result<Self, PasswordError> {
        let password = password.into();
        
        if password.is_empty() {
            return Err(PasswordError::Empty);
        }
        
        if password.len() < 8 {
            return Err(PasswordError::TooShort { min_length: 8 });
        }
        
        if password.len() > 128 {
            return Err(PasswordError::TooLong { max_length: 128 });
        }
        
        // Validar complejidad
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let _has_special = password.chars().any(|c| !c.is_alphanumeric());
        
        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(PasswordError::WeakPassword {
                needs_uppercase: !has_uppercase,
                needs_lowercase: !has_lowercase,
                needs_digit: !has_digit,
                needs_special: false, // No requerimos especiales por ahora
            });
        }
        
        Ok(Self(password))
    }
    
    /// Crear sin validación (para casos especiales como migración)
    pub fn new_unchecked(password: impl Into<String>) -> Self {
        Self(password.into())
    }
    
    /// Obtener el valor de la contraseña
    pub fn value(&self) -> &str {
        &self.0
    }
    
    /// Calcular fortaleza de la contraseña (0-100)
    pub fn strength(&self) -> u8 {
        let mut score = 0u8;
        let len = self.0.len();
        
        // Longitud
        score += match len {
            0..=7 => 0,
            8..=11 => 20,
            12..=15 => 30,
            _ => 40,
        };
        
        // Variedad de caracteres
        if self.0.chars().any(|c| c.is_uppercase()) {
            score += 15;
        }
        if self.0.chars().any(|c| c.is_lowercase()) {
            score += 15;
        }
        if self.0.chars().any(|c| c.is_ascii_digit()) {
            score += 15;
        }
        if self.0.chars().any(|c| !c.is_alphanumeric()) {
            score += 15;
        }
        
        score.min(100)
    }
}

// No implementamos Debug para evitar fugas de contraseña en logs
impl fmt::Debug for PlainPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlainPassword(***)")
    }
}

/// Errores de validación de contraseña
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordError {
    Empty,
    TooShort { min_length: usize },
    TooLong { max_length: usize },
    WeakPassword {
        needs_uppercase: bool,
        needs_lowercase: bool,
        needs_digit: bool,
        needs_special: bool,
    },
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordError::Empty => write!(f, "Password cannot be empty"),
            PasswordError::TooShort { min_length } => {
                write!(f, "Password must be at least {} characters", min_length)
            }
            PasswordError::TooLong { max_length } => {
                write!(f, "Password cannot exceed {} characters", max_length)
            }
            PasswordError::WeakPassword {
                needs_uppercase,
                needs_lowercase,
                needs_digit,
                needs_special,
            } => {
                let mut requirements = Vec::new();
                if *needs_uppercase {
                    requirements.push("uppercase letter");
                }
                if *needs_lowercase {
                    requirements.push("lowercase letter");
                }
                if *needs_digit {
                    requirements.push("digit");
                }
                if *needs_special {
                    requirements.push("special character");
                }
                write!(f, "Password must contain: {}", requirements.join(", "))
            }
        }
    }
}

impl std::error::Error for PasswordError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_password() {
        assert!(PlainPassword::new("SecurePass123").is_ok());
        assert!(PlainPassword::new("MyP@ssw0rd!").is_ok());
    }
    
    #[test]
    fn test_invalid_password() {
        assert!(PlainPassword::new("").is_err());
        assert!(PlainPassword::new("short").is_err());
        assert!(PlainPassword::new("alllowercase123").is_err());
        assert!(PlainPassword::new("ALLUPPERCASE123").is_err());
        assert!(PlainPassword::new("NoDigitsHere").is_err());
    }
    
    #[test]
    fn test_password_strength() {
        let weak = PlainPassword::new_unchecked("password");
        let medium = PlainPassword::new_unchecked("Password1");
        let strong = PlainPassword::new_unchecked("SecureP@ssw0rd!123");
        
        assert!(weak.strength() < medium.strength());
        assert!(medium.strength() < strong.strength());
    }
}
