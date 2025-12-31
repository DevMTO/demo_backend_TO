//! Notification Models
//! Modelos Diesel para notifications y notification_users

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::{
    Notification, NewNotification,
    NotificationUser, NewNotificationUser,
    NotificationWithReadStatus,
};
use crate::infrastructure::persistence::schema::{notifications, notification_users};

// ===== Notification Model =====

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationModel {
    pub id: i32,
    pub notification_type: String,
    pub category: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub metadata: Option<JsonValue>,
    pub priority: String,
    pub target_roles: Option<JsonValue>,
    pub target_user_id: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = notifications)]
pub struct NewNotificationModel {
    pub notification_type: String,
    pub category: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub metadata: Option<JsonValue>,
    pub priority: String,
    pub target_roles: Option<JsonValue>,
    pub target_user_id: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<i32>,
}

// ===== NotificationUser Model =====

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = notification_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationUserModel {
    pub id: i32,
    pub notification_id: i32,
    pub user_id: i32,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = notification_users)]
pub struct NewNotificationUserModel {
    pub notification_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = notification_users)]
pub struct UpdateNotificationUserModel {
    pub is_read: Option<bool>,
    pub read_at: Option<Option<DateTime<Utc>>>,
    pub is_dismissed: Option<bool>,
    pub dismissed_at: Option<Option<DateTime<Utc>>>,
}

// ===== Conversiones =====

impl From<NotificationModel> for Notification {
    fn from(model: NotificationModel) -> Self {
        Self {
            id: model.id,
            notification_type: model.notification_type,
            category: model.category,
            title: model.title,
            message: model.message,
            entity_type: model.entity_type,
            entity_id: model.entity_id,
            metadata: model.metadata,
            priority: model.priority,
            target_roles: model.target_roles,
            target_user_id: model.target_user_id,
            expires_at: model.expires_at,
            created_at: model.created_at,
            created_by: model.created_by,
        }
    }
}

impl From<NewNotification> for NewNotificationModel {
    fn from(n: NewNotification) -> Self {
        Self {
            notification_type: n.notification_type,
            category: n.category,
            title: n.title,
            message: n.message,
            entity_type: n.entity_type,
            entity_id: n.entity_id,
            metadata: n.metadata,
            priority: n.priority,
            target_roles: n.target_roles,
            target_user_id: n.target_user_id,
            expires_at: n.expires_at,
            created_by: n.created_by,
        }
    }
}

impl From<NotificationUserModel> for NotificationUser {
    fn from(model: NotificationUserModel) -> Self {
        Self {
            id: model.id,
            notification_id: model.notification_id,
            user_id: model.user_id,
            is_read: model.is_read,
            read_at: model.read_at,
            is_dismissed: model.is_dismissed,
            dismissed_at: model.dismissed_at,
            created_at: model.created_at,
        }
    }
}

impl From<NewNotificationUser> for NewNotificationUserModel {
    fn from(n: NewNotificationUser) -> Self {
        Self {
            notification_id: n.notification_id,
            user_id: n.user_id,
        }
    }
}

/// Tipo para resultado de join notification + notification_user
#[derive(Debug, Clone, Queryable)]
pub struct NotificationWithUserData {
    // Notification fields
    pub id: i32,
    pub notification_type: String,
    pub category: String,
    pub title: String,
    pub message: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub metadata: Option<JsonValue>,
    pub priority: String,
    pub target_roles: Option<JsonValue>,
    pub target_user_id: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    // NotificationUser fields
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
}

impl From<NotificationWithUserData> for NotificationWithReadStatus {
    fn from(data: NotificationWithUserData) -> Self {
        Self {
            notification: Notification {
                id: data.id,
                notification_type: data.notification_type,
                category: data.category,
                title: data.title,
                message: data.message,
                entity_type: data.entity_type,
                entity_id: data.entity_id,
                metadata: data.metadata,
                priority: data.priority,
                target_roles: data.target_roles,
                target_user_id: data.target_user_id,
                expires_at: data.expires_at,
                created_at: data.created_at,
                created_by: data.created_by,
            },
            is_read: data.is_read,
            read_at: data.read_at,
            is_dismissed: data.is_dismissed,
        }
    }
}
