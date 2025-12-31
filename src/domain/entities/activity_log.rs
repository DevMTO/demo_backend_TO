//! Activity Log Entity
//! Registro de actividades del sistema

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Tipos de acción del sistema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Auth,    // Acciones de autenticación
    Crud,    // Operaciones CRUD
    System,  // Acciones del sistema
}

impl ActionType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Auth => "auth",
            ActionType::Crud => "crud",
            ActionType::System => "system",
        }
    }
}

impl From<&str> for ActionType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "auth" => ActionType::Auth,
            "crud" => ActionType::Crud,
            "system" => ActionType::System,
            _ => ActionType::System,
        }
    }
}

/// Acciones específicas del sistema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Auth actions
    Login,
    Logout,
    LoginFailed,
    PasswordChanged,
    SessionExpired,
    
    // CRUD actions
    Create,
    Read,
    Update,
    Delete,
    List,
    
    // System actions
    Export,
    Import,
    Backup,
}

impl Action {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Login => "login",
            Action::Logout => "logout",
            Action::LoginFailed => "login_failed",
            Action::PasswordChanged => "password_changed",
            Action::SessionExpired => "session_expired",
            Action::Create => "create",
            Action::Read => "read",
            Action::Update => "update",
            Action::Delete => "delete",
            Action::List => "list",
            Action::Export => "export",
            Action::Import => "import",
            Action::Backup => "backup",
        }
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "login" => Action::Login,
            "logout" => Action::Logout,
            "login_failed" => Action::LoginFailed,
            "password_changed" => Action::PasswordChanged,
            "session_expired" => Action::SessionExpired,
            "create" => Action::Create,
            "read" => Action::Read,
            "update" => Action::Update,
            "delete" => Action::Delete,
            "list" => Action::List,
            "export" => Action::Export,
            "import" => Action::Import,
            "backup" => Action::Backup,
            _ => Action::Read,
        }
    }
}

/// Estado del log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LogStatus {
    Success,
    Error,
    Warning,
}

impl LogStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogStatus::Success => "success",
            LogStatus::Error => "error",
            LogStatus::Warning => "warning",
        }
    }
}

impl From<&str> for LogStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "success" => LogStatus::Success,
            "error" => LogStatus::Error,
            "warning" => LogStatus::Warning,
            _ => LogStatus::Success,
        }
    }
}

/// Tipos de entidades del sistema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    User,
    Persona,
    Agencia,
    Tour,
    Transporte,
    Vehiculo,
    Conductor,
    Guia,
    Restaurante,
    Entrada,
    File,
    Pago,
    Session,
    Notification,
    System,
}

impl EntityType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::User => "users",
            EntityType::Persona => "personas",
            EntityType::Agencia => "agencias",
            EntityType::Tour => "tours",
            EntityType::Transporte => "transportes",
            EntityType::Vehiculo => "vehiculos",
            EntityType::Conductor => "conductores",
            EntityType::Guia => "guias",
            EntityType::Restaurante => "restaurantes",
            EntityType::Entrada => "entradas",
            EntityType::File => "files",
            EntityType::Pago => "pagos",
            EntityType::Session => "sessions",
            EntityType::Notification => "notifications",
            EntityType::System => "system",
        }
    }
}

impl From<&str> for EntityType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "users" | "user" => EntityType::User,
            "personas" | "persona" => EntityType::Persona,
            "agencias" | "agencia" => EntityType::Agencia,
            "tours" | "tour" => EntityType::Tour,
            "transportes" | "transporte" => EntityType::Transporte,
            "vehiculos" | "vehiculo" => EntityType::Vehiculo,
            "conductores" | "conductor" => EntityType::Conductor,
            "guias" | "guia" => EntityType::Guia,
            "restaurantes" | "restaurante" => EntityType::Restaurante,
            "entradas" | "entrada" => EntityType::Entrada,
            "files" | "file" => EntityType::File,
            "pagos" | "pago" => EntityType::Pago,
            "sessions" | "session" => EntityType::Session,
            "notifications" | "notification" => EntityType::Notification,
            _ => EntityType::System,
        }
    }
}

/// Entidad Activity Log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
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

/// Builder para crear ActivityLog
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ActivityLogBuilder {
    user_id: Option<i32>,
    username: Option<String>,
    action_type: ActionType,
    action: Action,
    entity_type: EntityType,
    entity_id: Option<i32>,
    description: Option<String>,
    old_values: Option<JsonValue>,
    new_values: Option<JsonValue>,
    changed_fields: Option<Vec<String>>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    status: LogStatus,
    error_message: Option<String>,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::System
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Read
    }
}

impl Default for EntityType {
    fn default() -> Self {
        EntityType::System
    }
}

impl Default for LogStatus {
    fn default() -> Self {
        LogStatus::Success
    }
}

#[allow(dead_code)]
impl ActivityLogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user(mut self, user_id: i32, username: impl Into<String>) -> Self {
        self.user_id = Some(user_id);
        self.username = Some(username.into());
        self
    }

    pub fn action_type(mut self, action_type: ActionType) -> Self {
        self.action_type = action_type;
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    pub fn entity(mut self, entity_type: EntityType, entity_id: Option<i32>) -> Self {
        self.entity_type = entity_type;
        self.entity_id = entity_id;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn old_values(mut self, values: JsonValue) -> Self {
        self.old_values = Some(values);
        self
    }

    pub fn new_values(mut self, values: JsonValue) -> Self {
        self.new_values = Some(values);
        self
    }

    pub fn changed_fields(mut self, fields: Vec<String>) -> Self {
        self.changed_fields = Some(fields);
        self
    }

    pub fn request_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip;
        self.user_agent = user_agent;
        self
    }

    pub fn status(mut self, status: LogStatus) -> Self {
        self.status = status;
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.status = LogStatus::Error;
        self.error_message = Some(message.into());
        self
    }

    /// Construye el input para crear el log
    pub fn build(self) -> NewActivityLog {
        NewActivityLog {
            user_id: self.user_id,
            username: self.username,
            action_type: self.action_type.as_str().to_string(),
            action: self.action.as_str().to_string(),
            entity_type: self.entity_type.as_str().to_string(),
            entity_id: self.entity_id,
            description: self.description,
            old_values: self.old_values,
            new_values: self.new_values,
            changed_fields: self.changed_fields.map(|f| serde_json::json!(f)),
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            status: self.status.as_str().to_string(),
            error_message: self.error_message,
        }
    }
}

/// Estructura para crear nuevo ActivityLog
#[derive(Debug, Clone)]
pub struct NewActivityLog {
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
