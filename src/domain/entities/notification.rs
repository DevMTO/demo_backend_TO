//! Notification Entities
//! Sistema de notificaciones para usuarios

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Tipo de notificación
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    System,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
            NotificationType::Success => "success",
            NotificationType::System => "system",
        }
    }
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => NotificationType::Info,
            "warning" => NotificationType::Warning,
            "error" => NotificationType::Error,
            "success" => NotificationType::Success,
            "system" => NotificationType::System,
            _ => NotificationType::Info,
        }
    }
}

impl Default for NotificationType {
    fn default() -> Self {
        NotificationType::Info
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NotificationType::from(s))
    }
}

/// Categoría de notificación
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    Auth,
    Crud,
    System,
    Alert,
}

impl NotificationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationCategory::Auth => "auth",
            NotificationCategory::Crud => "crud",
            NotificationCategory::System => "system",
            NotificationCategory::Alert => "alert",
        }
    }
}

impl From<&str> for NotificationCategory {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "auth" => NotificationCategory::Auth,
            "crud" => NotificationCategory::Crud,
            "system" => NotificationCategory::System,
            "alert" => NotificationCategory::Alert,
            _ => NotificationCategory::System,
        }
    }
}

impl Default for NotificationCategory {
    fn default() -> Self {
        NotificationCategory::System
    }
}

impl std::str::FromStr for NotificationCategory {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NotificationCategory::from(s))
    }
}

/// Prioridad de notificación
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Urgent => "urgent",
        }
    }
}

impl From<&str> for NotificationPriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => NotificationPriority::Low,
            "normal" => NotificationPriority::Normal,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Normal,
        }
    }
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

impl std::str::FromStr for NotificationPriority {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NotificationPriority::from(s))
    }
}

/// Entidad Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
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

/// Entidad NotificationUser (relación usuario-notificación)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationUser {
    pub id: i32,
    pub notification_id: i32,
    pub user_id: i32,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Builder para crear Notification
#[derive(Debug, Clone, Default)]
pub struct NotificationBuilder {
    notification_type: NotificationType,
    category: NotificationCategory,
    title: String,
    message: String,
    entity_type: Option<String>,
    entity_id: Option<i32>,
    metadata: Option<JsonValue>,
    priority: NotificationPriority,
    target_roles: Option<Vec<String>>,
    target_user_id: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
    created_by: Option<i32>,
}

impl NotificationBuilder {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn notification_type(mut self, t: NotificationType) -> Self {
        self.notification_type = t;
        self
    }

    pub fn category(mut self, c: NotificationCategory) -> Self {
        self.category = c;
        self
    }

    #[allow(dead_code)]
    pub fn entity(mut self, entity_type: impl Into<String>, entity_id: i32) -> Self {
        self.entity_type = Some(entity_type.into());
        self.entity_id = Some(entity_id);
        self
    }

    #[allow(dead_code)]
    pub fn metadata(mut self, data: JsonValue) -> Self {
        self.metadata = Some(data);
        self
    }

    pub fn priority(mut self, p: NotificationPriority) -> Self {
        self.priority = p;
        self
    }

    pub fn for_roles(mut self, roles: Vec<String>) -> Self {
        self.target_roles = Some(roles);
        self
    }

    pub fn for_user(mut self, user_id: i32) -> Self {
        self.target_user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    pub fn expires_at(mut self, expires: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }

    pub fn created_by(mut self, user_id: i32) -> Self {
        self.created_by = Some(user_id);
        self
    }

    pub fn build(self) -> NewNotification {
        NewNotification {
            notification_type: self.notification_type.as_str().to_string(),
            category: self.category.as_str().to_string(),
            title: self.title,
            message: self.message,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            metadata: self.metadata,
            priority: self.priority.as_str().to_string(),
            target_roles: self.target_roles.map(|r| serde_json::json!(r)),
            target_user_id: self.target_user_id,
            expires_at: self.expires_at,
            created_by: self.created_by,
        }
    }
}

/// Estructura para crear nueva Notification
#[derive(Debug, Clone)]
pub struct NewNotification {
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

/// Estructura para crear NotificationUser
#[derive(Debug, Clone)]
pub struct NewNotificationUser {
    pub notification_id: i32,
    pub user_id: i32,
}

/// Notificación con datos de usuario para listados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationWithReadStatus {
    #[serde(flatten)]
    pub notification: Notification,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
}
