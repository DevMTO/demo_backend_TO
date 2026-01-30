//! Query parameters para Activity Log

use serde::Deserialize;

/// Parámetros de consulta para listar logs
#[derive(Debug, Deserialize)]
pub struct ListLogsParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filtrar por tipo de acción (auth, crud, system)
    pub action_type: Option<String>,
    /// Filtrar por acción específica (login, logout, create, update, delete, etc.)
    pub action: Option<String>,
    /// Filtrar por tipo de entidad (user, agencia, tour, etc.)
    pub entity_type: Option<String>,
    /// Filtrar por ID de entidad
    pub entity_id: Option<i32>,
    /// Filtrar por ID de usuario que realizó la acción
    pub user_id: Option<i32>,
    /// Filtrar por estado (success, failed, pending)
    pub status: Option<String>,
    /// Filtrar desde fecha (ISO 8601)
    pub from_date: Option<String>,
    /// Filtrar hasta fecha (ISO 8601)
    pub to_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LimitParam {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupParams {
    pub older_than_days: Option<i64>,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 50 }
