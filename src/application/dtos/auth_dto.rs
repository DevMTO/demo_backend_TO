//! # Auth DTOs
//! 
//! Data Transfer Objects para autenticación con cookies de sesión ultra-seguras.
//! NO usamos JWT - solo tokens de sesión opacos con HMAC.


use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request para login
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Identifier is required"))]
    pub identifier: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    
    /// Código MFA (opcional, requerido si MFA está habilitado)
    pub mfa_code: Option<String>,
    
    /// Recordar sesión (extender duración)
    #[serde(default)]
    pub remember_me: bool,
}

/// Request para registro
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub password_confirm: String,
    
    pub display_name: Option<String>,
    
    /// Documento de identidad (opcional)
    pub document: Option<DocumentInfo>,
}

/// Información de documento de identidad
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DocumentInfo {
    #[validate(length(min = 1, message = "Document type is required"))]
    pub document_type: String,
    
    #[validate(length(min = 1, message = "Document number is required"))]
    pub document_number: String,
}

/// Response de autenticación exitosa (solo sesión, sin JWT)
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub session_id: Uuid,
    pub expires_in: i64,
    /// Indica si la sesión fue extendida (remember_me)
    pub extended_session: bool,
}

impl AuthResponse {
    pub fn new(user: UserInfo, session_id: Uuid, expires_in: i64, extended_session: bool) -> Self {
        Self {
            user,
            session_id,
            expires_in,
            extended_session,
        }
    }
}

/// Información básica del usuario (sin datos sensibles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub mfa_enabled: bool,
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
