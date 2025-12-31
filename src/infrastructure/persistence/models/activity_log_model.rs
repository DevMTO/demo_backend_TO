//! Activity Log Model
//! Modelo Diesel para activity_logs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::{ActivityLog, NewActivityLog};
use crate::infrastructure::persistence::schema::activity_logs;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = activity_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ActivityLogModel {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub action_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i32>,
    pub description: Option<String>,
    pub old_values: Option<JsonValue>,
    pub new_values: Option<JsonValue>,
    pub changed_fields: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = activity_logs)]
pub struct NewActivityLogModel {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub action_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i32>,
    pub description: Option<String>,
    pub old_values: Option<JsonValue>,
    pub new_values: Option<JsonValue>,
    pub changed_fields: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}

// Conversiones

impl From<ActivityLogModel> for ActivityLog {
    fn from(model: ActivityLogModel) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            username: model.username,
            action_type: model.action_type,
            action: model.action,
            entity_type: model.entity_type,
            entity_id: model.entity_id,
            description: model.description,
            old_values: model.old_values,
            new_values: model.new_values,
            changed_fields: model.changed_fields,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            status: model.status,
            error_message: model.error_message,
            created_at: model.created_at,
        }
    }
}

impl From<NewActivityLog> for NewActivityLogModel {
    fn from(log: NewActivityLog) -> Self {
        Self {
            user_id: log.user_id,
            username: log.username,
            action_type: log.action_type,
            action: log.action,
            entity_type: log.entity_type,
            entity_id: log.entity_id,
            description: log.description,
            old_values: log.old_values,
            new_values: log.new_values,
            changed_fields: log.changed_fields,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            status: log.status,
            error_message: log.error_message,
        }
    }
}
