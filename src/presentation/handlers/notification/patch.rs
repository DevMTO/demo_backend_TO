//! PATCH handlers para Notification

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;

/// Marcar una notificación como leída
#[instrument(skip(state, auth))]
pub async fn mark_as_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(notification_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Marcando notificación {} como leída", notification_id);
    
    state.container.notification_service
        .mark_as_read(auth.user.id, notification_id)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "success": true,
        "message": "Notificación marcada como leída"
    })))
}

/// Marcar todas las notificaciones como leídas
#[instrument(skip(state, auth))]
pub async fn mark_all_as_read(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Marcando todas las notificaciones como leídas para usuario: {}", auth.user.id);
    
    let count = state.container.notification_service
        .mark_all_as_read(auth.user.id)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "success": true,
        "marked_count": count,
        "message": format!("{} notificaciones marcadas como leídas", count)
    })))
}

/// Descartar una notificación
#[instrument(skip(state, auth))]
pub async fn dismiss_notification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(notification_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚫 Descartando notificación {}", notification_id);
    
    state.container.notification_service
        .dismiss(auth.user.id, notification_id)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "success": true,
        "message": "Notificación descartada"
    })))
}

/// Descartar todas las notificaciones
#[instrument(skip(state, auth))]
pub async fn dismiss_all_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚫 Descartando todas las notificaciones para usuario: {}", auth.user.id);
    
    let count = state.container.notification_service
        .dismiss_all(auth.user.id)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "success": true,
        "dismissed_count": count,
        "message": format!("{} notificaciones descartadas", count)
    })))
}
