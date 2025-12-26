//! # Session Manager Port
//! 
//! Puerto de salida para gestión de sesiones ultra-seguras.
//! Reemplaza JWT con tokens opacos almacenados en BD.


use crate::domain::{entities::UserSession, errors::ApplicationError};

/// Token de sesión generado
#[derive(Debug, Clone)]
pub struct SessionTokenData {
    /// Token opaco (enviado al cliente en cookie)
    pub token: String,
    /// Hash del token (almacenado en BD)
    pub token_hash: String,
}

/// Puerto de salida para gestión de sesiones
pub trait SessionManagerPort: Send + Sync {
    /// Generar un nuevo token de sesión
    fn generate_token(&self) -> Result<SessionTokenData, ApplicationError>;
    
    /// Calcular hash de un token
    fn hash_token(&self, token: &str) -> Result<String, ApplicationError>;
    
    /// Verificar un token contra su hash (timing-safe)
    fn verify_token(&self, token: &str, stored_hash: &str) -> Result<bool, ApplicationError>;
    
    /// Crear una nueva sesión para un usuario
    fn create_session(
        &self,
        user_id: i32,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(UserSession, SessionTokenData), ApplicationError>;
    
    /// Validar que una sesión es válida (activa, no expirada, no idle)
    fn validate_session(&self, session: &UserSession) -> Result<(), ApplicationError>;
    
    /// Verificar si se debe rotar el token
    fn should_rotate_token(&self, session: &UserSession) -> bool;
    
    /// Rotar el token de sesión
    fn rotate_token(&self, session: &mut UserSession) -> Result<SessionTokenData, ApplicationError>;
    
    /// Actualizar timestamp de última actividad
    fn touch_session(&self, session: &mut UserSession);
}
