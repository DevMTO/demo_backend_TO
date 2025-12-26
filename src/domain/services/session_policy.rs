//! # Session Policy
//! 
//! Políticas de sesión y seguridad.


use chrono::Duration;

/// Políticas de sesión
#[derive(Debug, Clone)]
pub struct SessionPolicy {
    /// Duración del token de acceso
    pub access_token_duration: Duration,
    /// Duración del token de refresh
    pub refresh_token_duration: Duration,
    /// Máximo de sesiones activas por usuario
    pub max_sessions_per_user: usize,
    /// Permitir múltiples sesiones
    pub allow_multiple_sessions: bool,
    /// Requerir verificación de IP
    pub require_ip_verification: bool,
    /// Renovación automática de tokens
    pub auto_renew_tokens: bool,
    /// Tiempo antes de expiración para renovar
    pub renew_threshold: Duration,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self {
            access_token_duration: Duration::hours(24),
            refresh_token_duration: Duration::days(7),
            max_sessions_per_user: 5,
            allow_multiple_sessions: true,
            require_ip_verification: false,
            auto_renew_tokens: true,
            renew_threshold: Duration::hours(1),
        }
    }
}

impl SessionPolicy {
    /// Política estricta para alta seguridad
    pub fn strict() -> Self {
        Self {
            access_token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(1),
            max_sessions_per_user: 1,
            allow_multiple_sessions: false,
            require_ip_verification: true,
            auto_renew_tokens: false,
            renew_threshold: Duration::minutes(15),
        }
    }
    
    /// Política relajada para desarrollo
    pub fn relaxed() -> Self {
        Self {
            access_token_duration: Duration::days(7),
            refresh_token_duration: Duration::days(30),
            max_sessions_per_user: 10,
            allow_multiple_sessions: true,
            require_ip_verification: false,
            auto_renew_tokens: true,
            renew_threshold: Duration::hours(12),
        }
    }
    
    /// Obtener duración del token de acceso
    pub fn access_token_duration(&self) -> Duration {
        self.access_token_duration
    }
    
    /// Obtener duración del token de refresh
    pub fn refresh_token_duration(&self) -> Duration {
        self.refresh_token_duration
    }
}
