use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

use crate::application::ports::PasswordHasherPort;
use crate::domain::errors::ApplicationError;

pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    /// Crear con parámetros personalizados
    pub fn with_params(memory_size: u32, iterations: u32, parallelism: u32) -> Result<Self, ApplicationError> {
        let params = Params::new(memory_size, iterations, parallelism, None)
            .map_err(|e| ApplicationError::Configuration(format!("Invalid Argon2 params: {}", e)))?;
        
        Ok(Self {
            argon2: Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params),
        })
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasherPort for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ApplicationError> {
        let salt = SaltString::generate(&mut OsRng);
        
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ApplicationError::PasswordHashing(e.to_string()))
    }
    
    fn verify(&self, password: &str, hash: &str) -> Result<bool, ApplicationError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| ApplicationError::PasswordHashing(format!("Invalid hash format: {}", e)))?;
        
        Ok(self.argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_and_verify() {
        let hasher = Argon2PasswordHasher::new();
        let password = "SecurePassword123!";
        
        let hash = hasher.hash(password).unwrap();
        
        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }
    
    #[test]
    fn test_different_hashes_for_same_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = "SecurePassword123!";
        
        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();
        
        // Cada hash debe ser diferente (diferentes salts)
        assert_ne!(hash1, hash2);
        
        // Pero ambos deben verificar correctamente
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }
}
