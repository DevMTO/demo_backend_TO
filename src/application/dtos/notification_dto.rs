use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;

use crate::domain::entities::{NotificationWithReadStatus};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserNotificationDto {
    pub id: i32,
    pub notification_type: String,
    pub category: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    #[ts(type = "Record<string, any> | null")]
    pub metadata: Option<JsonValue>,
    pub priority: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
    #[ts(type = "string | null")]
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
}

impl From<NotificationWithReadStatus> for UserNotificationDto {
    fn from(n: NotificationWithReadStatus) -> Self {
        Self {
            id: n.notification.id,
            notification_type: n.notification.notification_type,
            category: n.notification.category,
            title: n.notification.title,
            message: n.notification.message,
            entity_type: n.notification.entity_type,
            entity_id: n.notification.entity_id,
            metadata: n.notification.metadata,
            priority: n.notification.priority,
            created_at: n.notification.created_at,
            is_read: n.is_read,
            read_at: n.read_at,
            is_dismissed: n.is_dismissed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateNotificationRequest {
    #[serde(default = "default_notification_type")]
    pub notification_type: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[validate(length(min = 1, max = 200, message = "El título debe tener entre 1 y 200 caracteres"))]
    pub title: String,
    #[validate(length(min = 1, max = 2000, message = "El mensaje debe tener entre 1 y 2000 caracteres"))]
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    #[ts(type = "Record<string, any> | null")]
    pub metadata: Option<JsonValue>,
    #[serde(default = "default_priority")]
    pub priority: String,
    pub target_roles: Option<Vec<String>>,
    pub target_user_id: Option<i32>,
    #[ts(type = "string | null")]
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_notification_type() -> String { "info".to_string() }
fn default_category() -> String { "system".to_string() }
fn default_priority() -> String { "normal".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserNotificationListDto {
    pub notifications: Vec<UserNotificationDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UnreadCountDto {
    pub unread_count: i64,
    pub user_id: i32,
}
