//! # Session Database Model
//! 
//! Modelo de Diesel para la tabla user_sessions (cookies ultra-seguras).

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::UserSession;
use crate::infrastructure::persistence::schema::user_sessions;

/// Modelo queryable para sesiones
/// El orden de los campos DEBE coincidir con el orden en schema.rs
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

/// Modelo insertable para sesiones
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_sessions)]
pub struct NewSessionModel<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: &'a str,
    pub refresh_token_hash: Option<&'a str>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub device_fingerprint: Option<&'a str>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<&'a str>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

/// Modelo actualizable para sesiones
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_sessions)]
pub struct UpdateSessionModel<'a> {
    pub token_hash: Option<&'a str>,
    pub refresh_token_hash: Option<Option<&'a str>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_expires_at: Option<Option<DateTime<Utc>>>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: Option<Option<DateTime<Utc>>>,
    pub is_active: Option<bool>,
    pub revoked_at: Option<Option<DateTime<Utc>>>,
    pub revoked_reason: Option<Option<&'a str>>,
}

// Conversiones

impl From<SessionModel> for UserSession {
    fn from(model: SessionModel) -> Self {
        UserSession {
            id: model.id,
            user_id: model.user_id,
            token_hash: model.token_hash,
            refresh_token_hash: model.refresh_token_hash,
            expires_at: model.expires_at,
            refresh_expires_at: model.refresh_expires_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            device_fingerprint: model.device_fingerprint,
            is_active: model.is_active,
            revoked_at: model.revoked_at,
            revoked_reason: model.revoked_reason,
            last_activity_at: model.last_activity_at,
        }
    }
}

impl<'a> From<&'a UserSession> for NewSessionModel<'a> {
    fn from(session: &'a UserSession) -> Self {
        NewSessionModel {
            id: session.id,
            user_id: session.user_id,
            token_hash: &session.token_hash,
            refresh_token_hash: session.refresh_token_hash.as_deref(),
            expires_at: session.expires_at,
            refresh_expires_at: session.refresh_expires_at,
            created_at: session.created_at,
            updated_at: session.updated_at,
            ip_address: session.ip_address.as_deref(),
            user_agent: session.user_agent.as_deref(),
            device_fingerprint: session.device_fingerprint.as_deref(),
            is_active: session.is_active,
            revoked_at: session.revoked_at,
            revoked_reason: session.revoked_reason.as_deref(),
            last_activity_at: session.last_activity_at,
        }
    }
}
