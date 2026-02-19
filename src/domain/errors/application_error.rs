use super::DomainError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationError {
    /// Error del dominio
    Domain(DomainError),
    
    /// Error de repositorio/persistencia
    Repository(String),
    
    /// Error de autenticación
    Authentication(String),
    
    /// Error de autorización
    Authorization(String),
    
    /// Validación de entrada fallida
    ValidationFailed(Vec<String>),
    
    /// Error de validación simple
    Validation(String),
    
    /// Error de configuración
    Configuration(String),
    
    /// Error generando token
    TokenGeneration(String),
    
    /// Error validando token
    TokenValidation(String),
    
    /// Error hasheando contraseña
    PasswordHashing(String),
    
    /// Error criptográfico (HMAC, etc.)
    Cryptographic(String),
    
    /// Error interno del servidor
    InternalError(String),
    
    /// Solicitud inválida
    BadRequest(String),
    
    /// Acceso prohibido
    Forbidden(String),
    
    /// Recurso no encontrado
    NotFound(String),
    
    /// Conflicto (ej: duplicado)
    Conflict(String),
    
    /// Rate limit excedido
    RateLimitExceeded,
    
    /// Sesión requerida
    SessionRequired,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::Domain(err) => write!(f, "{}", err),
            ApplicationError::Repository(msg) => write!(f, "Error de base de datos: {}", msg),
            ApplicationError::Authentication(msg) => write!(f, "Error de autenticación: {}", msg),
            ApplicationError::Authorization(msg) => write!(f, "Error de autorización: {}", msg),
            ApplicationError::ValidationFailed(errors) => {
                write!(f, "Validación fallida: {}", errors.join(", "))
            }
            ApplicationError::Validation(msg) => write!(f, "Error de validación: {}", msg),
            ApplicationError::Configuration(msg) => write!(f, "Error de configuración: {}", msg),
            ApplicationError::TokenGeneration(msg) => write!(f, "Error generando token: {}", msg),
            ApplicationError::TokenValidation(msg) => write!(f, "Token inválido: {}", msg),
            ApplicationError::PasswordHashing(msg) => write!(f, "Error de seguridad: {}", msg),
            ApplicationError::Cryptographic(msg) => write!(f, "Error criptográfico: {}", msg),
            ApplicationError::InternalError(msg) => write!(f, "Error interno: {}", msg),
            ApplicationError::BadRequest(msg) => write!(f, "Solicitud inválida: {}", msg),
            ApplicationError::Forbidden(msg) => write!(f, "Acceso denegado: {}", msg),
            ApplicationError::NotFound(msg) => write!(f, "No encontrado: {}", msg),
            ApplicationError::Conflict(msg) => write!(f, "Conflicto: {}", msg),
            ApplicationError::RateLimitExceeded => write!(f, "Demasiadas solicitudes, intente más tarde"),
            ApplicationError::SessionRequired => write!(f, "Sesión requerida"),
        }
    }
}

impl std::error::Error for ApplicationError {}

impl From<DomainError> for ApplicationError {
    fn from(err: DomainError) -> Self {
        ApplicationError::Domain(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            ApplicationError::Domain(DomainError::EntityNotFound { .. }) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Domain(DomainError::InactiveUser)
            | ApplicationError::Domain(DomainError::EmailNotVerified)
            | ApplicationError::Domain(DomainError::InvalidToken)
            | ApplicationError::Domain(DomainError::SessionExpired) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: "UNAUTHORIZED".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Domain(DomainError::MfaRequired) => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    error: "MFA_REQUIRED".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Authentication(_) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: "AUTHENTICATION_ERROR".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Authorization(_) | ApplicationError::Forbidden(_) => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    error: "FORBIDDEN".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::ValidationFailed(errors) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "VALIDATION_ERROR".to_string(),
                    message: "Validation failed".to_string(),
                    details: Some(errors.clone()),
                },
            ),
            ApplicationError::Validation(_) | ApplicationError::BadRequest(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "BAD_REQUEST".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Conflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    error: "CONFLICT".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorResponse {
                    error: "RATE_LIMIT_EXCEEDED".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::SessionRequired | ApplicationError::TokenValidation(_) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: "UNAUTHORIZED".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApplicationError::Repository(ref msg) => {
                tracing::error!("Repository error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "REPOSITORY_ERROR".to_string(),
                        message: format!("Error de repositorio: {}", msg),
                        details: None,
                    },
                )
            },
            _ => {
                tracing::error!("Unhandled error: {:?}", &self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: "INTERNAL_ERROR".to_string(),
                        message: "Error interno del servidor".to_string(),
                        details: None,
                    },
                )
            },
        };
        
        (status, Json(error_response)).into_response()
    }
}
