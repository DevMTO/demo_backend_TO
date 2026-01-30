//! Query parameters para Notification

use serde::Deserialize;

/// Parámetros de consulta para listar notificaciones
#[derive(Debug, Deserialize)]
pub struct ListNotificationsParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filtrar solo no leídas
    pub unread_only: Option<bool>,
    /// Filtrar por tipo (info, success, warning, error)
    pub notification_type: Option<String>,
    /// Filtrar por categoría (system, auth, crud, business)
    pub category: Option<String>,
}

/// Parámetros para cleanup de notificaciones
#[derive(Debug, Deserialize)]
pub struct CleanupParams {
    /// Días para mantener notificaciones de prioridad baja (default: 7)
    #[serde(default = "default_days_low")]
    pub days_low: i32,
    /// Días para mantener notificaciones de prioridad normal (default: 14)
    #[serde(default = "default_days_normal")]
    pub days_normal: i32,
    /// Días para mantener notificaciones de prioridad alta (default: 30)
    #[serde(default = "default_days_high")]
    pub days_high: i32,
    /// Días para mantener notificaciones de prioridad urgente (default: 60)
    #[serde(default = "default_days_urgent")]
    pub days_urgent: i32,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }
fn default_days_low() -> i32 { 7 }
fn default_days_normal() -> i32 { 14 }
fn default_days_high() -> i32 { 30 }
fn default_days_urgent() -> i32 { 60 }
