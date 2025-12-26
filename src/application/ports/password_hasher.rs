//! # Password Hasher Port
//! 
//! Puerto de salida para hashing de contraseñas.

use crate::domain::errors::ApplicationError;

/// Puerto de salida para hashing de contraseñas
pub trait PasswordHasherPort: Send + Sync {
    /// Hashear una contraseña
    fn hash(&self, password: &str) -> Result<String, ApplicationError>;
    
    /// Verificar una contraseña contra un hash
    fn verify(&self, password: &str, hash: &str) -> Result<bool, ApplicationError>;
}
