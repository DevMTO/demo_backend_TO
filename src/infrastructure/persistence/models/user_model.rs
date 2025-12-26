//! # User Database Model
//! 
//! Modelo de Diesel para la tabla users según el diagrama.
//! IMPORTANTE: El orden de los campos DEBE coincidir con schema.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{User, UserRole, UserStatus};
use crate::infrastructure::persistence::schema::users;

/// Modelo queryable para usuarios
/// Orden de campos según schema.rs:
/// id, id_persona, username, email, password_hash, role, id_entidad, 
/// nombre_entidad, status, last_login, created_at, updated_at, created_by, updated_by
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub id_entidad: Option<i32>,
    pub nombre_entidad: Option<String>,
    pub status: String,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

/// Modelo insertable para usuarios (sin id, es SERIAL)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUserModel<'a> {
    pub id_persona: Option<i32>,
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
    pub id_entidad: Option<i32>,
    pub nombre_entidad: Option<&'a str>,
    pub status: &'a str,
    pub last_login: Option<DateTime<Utc>>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

/// Modelo actualizable para usuarios
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUserModel<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password_hash: Option<&'a str>,
    pub role: Option<&'a str>,
    pub id_entidad: Option<Option<i32>>,
    pub nombre_entidad: Option<Option<&'a str>>,
    pub status: Option<&'a str>,
    pub last_login: Option<Option<DateTime<Utc>>>,
    pub updated_by: Option<i32>,
}

// Conversiones entre modelos de dominio y persistencia

impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        User {
            id: model.id,
            id_persona: model.id_persona,
            username: model.username,
            email: model.email,
            password_hash: model.password_hash,
            role: model.role.parse().unwrap_or_default(),
            id_entidad: model.id_entidad,
            nombre_entidad: model.nombre_entidad,
            status: model.status.parse().unwrap_or_default(),
            last_login: model.last_login,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a User> for NewUserModel<'a> {
    fn from(user: &'a User) -> Self {
        NewUserModel {
            id_persona: user.id_persona,
            username: &user.username,
            email: &user.email,
            password_hash: &user.password_hash,
            role: match &user.role {
                UserRole::SuperAdmin => "superadmin",
                UserRole::Admin => "admin",
                UserRole::SubAdmin => "subadmin",
                UserRole::Operador => "operador",
                UserRole::Viewer => "viewer",
            },
            id_entidad: user.id_entidad,
            nombre_entidad: user.nombre_entidad.as_deref(),
            status: match &user.status {
                UserStatus::Activo => "activo",
                UserStatus::Inactivo => "inactivo",
                UserStatus::Suspendido => "suspendido",
                UserStatus::PendienteVerificacion => "pendiente_verificacion",
            },
            last_login: user.last_login,
            created_by: user.created_by,
            updated_by: user.updated_by,
        }
    }
}
