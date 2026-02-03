use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{debug, warn};

use crate::application::ports::{SessionManagerPort, SessionTokenData};
use crate::config::AppConfig;
use crate::domain::entities::UserSession;
use crate::domain::errors::ApplicationError;

type HmacSha256 = Hmac<Sha256>;

/// Periodo de gracia después de una rotación durante el cual NO se rota de nuevo.
/// Esto evita problemas con múltiples tabs abiertas que envían peticiones simultáneas.
const ROTATION_GRACE_PERIOD_SECS: i64 = 30;

#[derive(Clone)]
pub struct SecureSessionManager {
    secret_key: Vec<u8>,
    session_expiration_hours: i64,
    rotation_interval_minutes: i64,
}

impl SecureSessionManager {
    pub fn new(config: &AppConfig) -> Result<Self, ApplicationError> {
        if config.session_secret.len() < 64 {
            return Err(ApplicationError::Configuration(
                "SESSION_SECRET must be at least 64 characters".to_string(),
            ));
        }

        Ok(Self {
            secret_key: config.session_secret.as_bytes().to_vec(),
            session_expiration_hours: config.session_expiration_hours,
            rotation_interval_minutes: config.session_rotation_interval_minutes,
        })
    }

    fn is_expired(&self, session: &UserSession) -> bool {
        Utc::now() > session.expires_at
    }
}

impl SessionManagerPort for SecureSessionManager {
    fn generate_token(&self) -> Result<SessionTokenData, ApplicationError> {
        let mut token_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut token_bytes);
        let token = URL_SAFE_NO_PAD.encode(token_bytes);
        let token_hash = self.hash_token(&token)?;
        Ok(SessionTokenData { token, token_hash })
    }

    fn hash_token(&self, token: &str) -> Result<String, ApplicationError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .map_err(|e| ApplicationError::Cryptographic(format!("HMAC error: {}", e)))?;
        mac.update(token.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    fn verify_token(&self, token: &str, stored_hash: &str) -> Result<bool, ApplicationError> {
        let computed_hash = self.hash_token(token)?;
        let computed_bytes = computed_hash.as_bytes();
        let stored_bytes = stored_hash.as_bytes();
        Ok(computed_bytes.ct_eq(stored_bytes).into())
    }

    fn create_session(
        &self,
        user_id: i32,
        user_agent: Option<String>,
        ip_address: Option<String>,
        remember_me: bool,
    ) -> Result<(UserSession, SessionTokenData), ApplicationError> {
        let session_token = self.generate_token()?;
        let now = Utc::now();
        
        // Si remember_me, extender la duración de la sesión a 30 días
        let expiration_hours = if remember_me {
            30 * 24 // 30 días
        } else {
            self.session_expiration_hours
        };

        let session = UserSession {
            id: 0,
            user_id,
            token_hash: session_token.token_hash.clone(),
            refresh_token_hash: None,
            expires_at: now + Duration::hours(expiration_hours),
            refresh_expires_at: None,
            ip_address,
            user_agent,
            device_fingerprint: None,
            is_active: true,
            last_activity: Some(now),
            revoked_at: None,
            revoked_reason: None,
            created_at: now,
            updated_at: now,
            remember_me,
        };

        Ok((session, session_token))
    }

    fn validate_session(&self, session: &UserSession) -> Result<(), ApplicationError> {
        debug!(
            "Validando sesión {} - is_active: {}, expires_at: {}, last_activity: {:?}",
            session.id, session.is_active, session.expires_at, session.last_activity
        );
        
        if !session.is_active {
            warn!("Sesión {} no está activa (revocada o cerrada)", session.id);
            return Err(ApplicationError::Authentication("Session is not active".to_string()));
        }
        if self.is_expired(session) {
            warn!("Sesión {} ha expirado (expires_at: {})", session.id, session.expires_at);
            return Err(ApplicationError::Authentication("Session has expired".to_string()));
        }
        
        debug!("Sesión {} válida", session.id);
        Ok(())
    }

    fn should_rotate_token(&self, session: &UserSession) -> bool {
        // Verificar si pasó suficiente tiempo desde la última actividad
        if let Some(last_activity) = session.last_activity {
            let now = Utc::now();
            let time_since_activity = now - last_activity;
            let rotation_threshold = Duration::minutes(self.rotation_interval_minutes);
            
            // Si no ha pasado el tiempo de rotación, no rotar
            if time_since_activity <= rotation_threshold {
                return false;
            }
            
            // Periodo de gracia: Si la última actualización fue muy reciente
            // (otro request ya rotó), no rotar de nuevo.
            // Esto evita problemas con múltiples tabs enviando requests simultáneos.
            let time_since_update = now - session.updated_at;
            if time_since_update < Duration::seconds(ROTATION_GRACE_PERIOD_SECS) {
                debug!(
                    "Sesión {} dentro del periodo de gracia de rotación ({:?} desde última actualización)",
                    session.id, time_since_update
                );
                return false;
            }
            
            debug!(
                "Sesión {} necesita rotación de token (last_activity: {}, threshold: {} min)",
                session.id, last_activity, self.rotation_interval_minutes
            );
            true
        } else {
            false
        }
    }

    fn rotate_token(&self, session: &mut UserSession) -> Result<SessionTokenData, ApplicationError> {
        debug!("Rotando token para sesión {}", session.id);
        let new_token = self.generate_token()?;
        session.token_hash = new_token.token_hash.clone();
        session.updated_at = Utc::now();
        session.last_activity = Some(Utc::now());
        debug!("Token rotado exitosamente para sesión {}", session.id);
        Ok(new_token)
    }

    fn touch_session(&self, session: &mut UserSession) {
        session.last_activity = Some(Utc::now());
        session.updated_at = Utc::now();
    }
}
