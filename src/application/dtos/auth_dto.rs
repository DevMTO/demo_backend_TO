//! # Auth DTOs
//! 
//! Data Transfer Objects para autenticación con cookies de sesión.
//! NO usamos JWT - solo tokens de sesión opacos con HMAC.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request para login
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Identifier is required"))]
    pub identifier: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    
    /// Recordar sesión (extender duración)
    #[serde(default)]
    pub remember_me: bool,
}

/// Request para registro de usuario
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    /// ID de la persona ya registrada en el sistema (opcional)
    pub id_persona: Option<i32>,
    
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub password_confirm: String,
    
    /// Rol del usuario (opcional, default: Operador)
    pub role: Option<String>,
    
    /// ID de la entidad asociada (agencia, transporte, etc.)
    pub id_entidad: Option<i32>,
    
    /// Nombre de la entidad asociada
    pub nombre_entidad: Option<String>,
}

/// Response de autenticación exitosa (solo sesión, sin JWT)
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user: AuthUserInfo,
    pub session_id: i32,
    pub expires_in: i64,
    /// Indica si la sesión fue extendida (remember_me)
    pub extended_session: bool,
}

impl AuthResponse {
    pub fn new(user: AuthUserInfo, session_id: i32, expires_in: i64, extended_session: bool) -> Self {
        Self {
            user,
            session_id,
            expires_in,
            extended_session,
        }
    }
}

/// Información del usuario autenticado (sin datos sensibles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserInfo {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<i32>,
    pub nombre_entidad: Option<String>,
    pub status: String,
}

/// Request para cambio de contraseña
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
    
    #[validate(must_match(other = "new_password", message = "Passwords do not match"))]
    pub new_password_confirm: String,
}

/// Request para logout
#[derive(Debug, Clone, Deserialize)]
pub struct LogoutRequest {
    /// Cerrar todas las sesiones
    #[serde(default)]
    pub all_sessions: bool,
}

/// Response genérica de éxito
#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }
}
