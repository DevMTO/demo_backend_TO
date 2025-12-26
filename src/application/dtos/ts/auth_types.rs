//! # Auth Types - TypeScript Exports
//!
//! Tipos de autenticación exportables a TypeScript.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::user_types::UserInfoTs;

/// Request para login
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct LoginRequestTs {
    pub identifier: String,
    pub password: String,
    pub mfa_code: Option<String>,
    #[serde(default)]
    pub remember_me: bool,
}

/// Response de autenticación exitosa
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct AuthResponseTs {
    pub user: UserInfoTs,
    pub session_id: Uuid,
    pub expires_in: i64,
    pub extended_session: bool,
}

/// Request para cambio de contraseña
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequestTs {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

/// Request para logout
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequestTs {
    #[serde(default)]
    pub all_sessions: bool,
}

/// Response genérica de éxito
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponseTs {
    pub success: bool,
    pub message: String,
}

/// Response de error estándar
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseTs {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
    pub details: Option<Vec<String>>,
}
