//! # User Database Model
//! 
//! Modelo de Diesel para la tabla users.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{User, UserRole};
use crate::infrastructure::persistence::schema::users;

/// Modelo queryable para usuarios
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: i32,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub mfa_backup_codes: Option<serde_json::Value>,
}

/// Modelo insertable para usuarios
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUserModel<'a> {
    pub id: Uuid,
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: Option<&'a str>,
    pub role: &'a str,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_by: Option<&'a str>,
    pub updated_by: Option<&'a str>,
    pub version: i32,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<&'a str>,
    pub mfa_backup_codes: Option<serde_json::Value>,
}

/// Modelo actualizable para usuarios
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUserModel<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
    pub display_name: Option<Option<&'a str>>,
    pub role: Option<&'a str>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<Option<DateTime<Utc>>>,
    pub updated_by: Option<Option<&'a str>>,
    pub version: Option<i32>,
    pub mfa_enabled: Option<bool>,
    pub mfa_secret: Option<Option<&'a str>>,
    pub mfa_backup_codes: Option<Option<serde_json::Value>>,
}

// Conversiones entre modelos de dominio y persistencia

impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        User {
            id: model.id,
            username: model.username,
            email: model.email,
            password_hash: model.password_hash,
            display_name: model.display_name,
            role: model.role.parse().unwrap_or_default(),
            email_verified: model.email_verified,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            last_login: model.last_login,
            created_by: model.created_by,
            updated_by: model.updated_by,
            version: model.version,
            mfa_enabled: model.mfa_enabled,
            mfa_secret: model.mfa_secret,
            mfa_backup_codes: model.mfa_backup_codes,
        }
    }
}

impl<'a> From<&'a User> for NewUserModel<'a> {
    fn from(user: &'a User) -> Self {
        NewUserModel {
            id: user.id,
            username: &user.username,
            email: &user.email,
            password_hash: &user.password_hash,
            display_name: user.display_name.as_deref(),
            role: match &user.role {
                UserRole::SuperAdmin => "superadmin",
                UserRole::Admin => "admin",
                UserRole::SubAdmin => "subadmin",
                UserRole::User => "user",
                UserRole::Viewer => "viewer",
            },
            email_verified: user.email_verified,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
            created_by: user.created_by.as_deref(),
            updated_by: user.updated_by.as_deref(),
            version: user.version,
            mfa_enabled: user.mfa_enabled,
            mfa_secret: user.mfa_secret.as_deref(),
            mfa_backup_codes: user.mfa_backup_codes.clone(),
        }
    }
}
