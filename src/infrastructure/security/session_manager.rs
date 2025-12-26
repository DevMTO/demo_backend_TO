//! # Secure Session Manager
//! 
//! Gestor de sesiones ultra-seguras usando tokens opacos y cookies HttpOnly.
//! No usa JWT - todas las sesiones se almacenan en la base de datos.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::application::ports::{SessionManagerPort, SessionTokenData};
use crate::config::AppConfig;
use crate::domain::entities::UserSession;
use crate::domain::errors::ApplicationError;

type HmacSha256 = Hmac<Sha256>;

/// Gestor de sesiones ultra-seguras
#[derive(Clone)]
pub struct SecureSessionManager {
    secret_key: Vec<u8>,
    session_expiration_hours: i64,
    idle_timeout_minutes: i64,
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
            idle_timeout_minutes: config.session_idle_timeout_minutes,
            rotation_interval_minutes: config.session_rotation_interval_minutes,
        })
    }

    fn is_idle_timeout(&self, session: &UserSession) -> bool {
        if let Some(last_activity) = session.last_activity_at {
            let idle_threshold = Duration::minutes(self.idle_timeout_minutes);
            Utc::now() - last_activity > idle_threshold
        } else {
            false
        }
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
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(UserSession, SessionTokenData), ApplicationError> {
        let session_token = self.generate_token()?;
        let now = Utc::now();

        let session = UserSession {
            id: Uuid::new_v4(),
            user_id,
            token_hash: session_token.token_hash.clone(),
            refresh_token_hash: None,
            created_at: now,
            updated_at: now,
            expires_at: now + Duration::hours(self.session_expiration_hours),
            refresh_expires_at: None,
            user_agent,
            ip_address,
            device_fingerprint: None, // TODO: Implementar fingerprinting
            is_active: true,
            revoked_at: None,
            revoked_reason: None,
            last_activity_at: Some(now),
        };

        Ok((session, session_token))
    }

    fn validate_session(&self, session: &UserSession) -> Result<(), ApplicationError> {
        if !session.is_active {
            return Err(ApplicationError::Authentication("Session is not active".to_string()));
        }
        if self.is_expired(session) {
            return Err(ApplicationError::Authentication("Session has expired".to_string()));
        }
        if self.is_idle_timeout(session) {
            return Err(ApplicationError::Authentication("Session idle timeout".to_string()));
        }
        Ok(())
    }

    fn should_rotate_token(&self, session: &UserSession) -> bool {
        if let Some(last_activity) = session.last_activity_at {
            let rotation_threshold = Duration::minutes(self.rotation_interval_minutes);
            Utc::now() - last_activity > rotation_threshold
        } else {
            false
        }
    }

    fn rotate_token(&self, session: &mut UserSession) -> Result<SessionTokenData, ApplicationError> {
        let new_token = self.generate_token()?;
        session.token_hash = new_token.token_hash.clone();
        session.updated_at = Utc::now();
        session.last_activity_at = Some(Utc::now());
        Ok(new_token)
    }

    fn touch_session(&self, session: &mut UserSession) {
        session.last_activity_at = Some(Utc::now());
        session.updated_at = Utc::now();
    }
}
