use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;

use crate::domain::entities::ActivityLog;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ActivityLogDto {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    /// Nombre completo de la persona que realizó la acción (si existe)
    pub user_full_name: Option<String>,
    pub action_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i32>,
    pub description: Option<String>,
    #[ts(type = "Record<string, any> | null")]
    pub old_values: Option<JsonValue>,
    #[ts(type = "Record<string, any> | null")]
    pub new_values: Option<JsonValue>,
    #[ts(type = "string[] | null")]
    pub changed_fields: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

impl From<ActivityLog> for ActivityLogDto {
    fn from(log: ActivityLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            username: log.username,
            user_full_name: None,
            action_type: log.action_type,
            action: log.action,
            entity_type: log.entity_type,
            entity_id: log.entity_id,
            description: log.description,
            old_values: log.old_values,
            new_values: log.new_values,
            changed_fields: log.changed_fields,
            ip_address: log.ip_address,
            status: log.status,
            error_message: log.error_message,
            created_at: log.created_at,
        }
    }
}

/// DTO para log con información de usuario enriquecida
#[derive(Debug, Clone)]
pub struct ActivityLogWithUser {
    pub log: ActivityLog,
    pub user_full_name: Option<String>,
}

impl From<ActivityLogWithUser> for ActivityLogDto {
    fn from(data: ActivityLogWithUser) -> Self {
        Self {
            id: data.log.id,
            user_id: data.log.user_id,
            username: data.log.username,
            user_full_name: data.user_full_name,
            action_type: data.log.action_type,
            action: data.log.action,
            entity_type: data.log.entity_type,
            entity_id: data.log.entity_id,
            description: data.log.description,
            old_values: data.log.old_values,
            new_values: data.log.new_values,
            changed_fields: data.log.changed_fields,
            ip_address: data.log.ip_address,
            status: data.log.status,
            error_message: data.log.error_message,
            created_at: data.log.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ActivityLogFilters {
    pub user_id: Option<i32>,
    pub action_type: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub status: Option<String>,
    #[ts(type = "string | null")]
    pub from_date: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ActivityLogListDto {
    pub logs: Vec<ActivityLogDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ActivityLogSummaryDto {
    pub total_logs: i64,
    pub by_action_type: Vec<ActionTypeSummary>,
    pub by_status: Vec<StatusSummary>,
    pub recent_errors: Vec<ActivityLogDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ActionTypeSummary {
    pub action_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct StatusSummary {
    pub status: String,
    pub count: i64,
}
