//! # Session Entity
//! 
//! Entidad de sesión de usuario para autenticación segura con cookies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entidad de Sesión de Usuario
/// 
/// Representa una sesión activa de un usuario en el sistema.
/// Utilizada con tokens opacos almacenados en BD (no JWT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// ID único de la sesión
    pub id: Uuid,
    /// ID del usuario
    pub user_id: Uuid,
    /// Hash del token de sesión (HMAC-SHA256)
    pub token_hash: String,
    /// Hash del token de refresh (opcional, no usado en sistema actual)
    pub refresh_token_hash: Option<String>,
    /// Fecha de expiración del token
    pub expires_at: DateTime<Utc>,
    /// Fecha de expiración del token de refresh
    pub refresh_expires_at: Option<DateTime<Utc>>,
    /// Fecha de creación de la sesión
    pub created_at: DateTime<Utc>,
    /// Fecha de última actualización
    pub updated_at: DateTime<Utc>,
    /// Última actividad del usuario (para idle timeout)
    pub last_activity_at: Option<DateTime<Utc>>,
    /// Dirección IP del cliente
    pub ip_address: Option<String>,
    /// User-Agent del navegador
    pub user_agent: Option<String>,
    /// Fingerprint del dispositivo
    pub device_fingerprint: Option<String>,
    /// Si la sesión está activa
    pub is_active: bool,
    /// Fecha de revocación (si aplica)
    pub revoked_at: Option<DateTime<Utc>>,
    /// Razón de revocación
    pub revoked_reason: Option<String>,
}

impl UserSession {
    /// Crear una nueva sesión
    pub fn new(
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            refresh_token_hash: None,
            expires_at,
            refresh_expires_at: None,
            created_at: now,
            updated_at: now,
            last_activity_at: Some(now),
            ip_address,
            user_agent,
            device_fingerprint: None,
            is_active: true,
            revoked_at: None,
            revoked_reason: None,
        }
    }
    
    /// Crear sesión con refresh token
    pub fn with_refresh_token(
        mut self,
        refresh_token_hash: String,
        refresh_expires_at: DateTime<Utc>,
    ) -> Self {
        self.refresh_token_hash = Some(refresh_token_hash);
        self.refresh_expires_at = Some(refresh_expires_at);
        self
    }
    
    /// Agregar fingerprint de dispositivo
    pub fn with_device_fingerprint(mut self, fingerprint: String) -> Self {
        self.device_fingerprint = Some(fingerprint);
        self
    }
    
    /// Verifica si la sesión ha expirado
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Verifica si la sesión es válida (activa y no expirada)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
    
    /// Verifica si el refresh token ha expirado
    pub fn is_refresh_expired(&self) -> bool {
        match self.refresh_expires_at {
            Some(exp) => Utc::now() > exp,
            None => true,
        }
    }
    
    /// Tiempo restante hasta la expiración en segundos
    pub fn seconds_until_expiry(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
    
    /// Verifica si la sesión está próxima a expirar (menos de 1 hora)
    pub fn is_near_expiry(&self) -> bool {
        let secs = self.seconds_until_expiry();
        secs < 3600 && secs > 0
    }
    
    /// Revocar la sesión
    pub fn revoke(&mut self, reason: String) {
        self.is_active = false;
        self.revoked_at = Some(Utc::now());
        self.revoked_reason = Some(reason);
        self.updated_at = Utc::now();
    }
    
    /// Actualizar el token de acceso
    pub fn update_access_token(&mut self, new_token_hash: String, new_expires_at: DateTime<Utc>) {
        self.token_hash = new_token_hash;
        self.expires_at = new_expires_at;
        self.updated_at = Utc::now();
    }
}
