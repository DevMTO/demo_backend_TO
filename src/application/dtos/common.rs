//! Common DTOs shared across modules

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// User information for audit/logging purposes
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AuditInfo {
    pub user_id: i32,
    pub username: String,
    pub is_admin: bool,
}
