//! DELETE handlers para Notification

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, json_deleted};
use crate::presentation::extractors::AuthUser;

use super::query_params::CleanupParams;

/// Eliminar una notificación (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn delete_notification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(notification_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede eliminar notificaciones".to_string()
        ));
    }
    
    info!("[DELETE] Eliminando notificación: {}", notification_id);
    
    let _ = state.container.notification_repository
        .find_by_id(notification_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(
            format!("Notificación {} no encontrada", notification_id)
        ))?;
    
    state.container.notification_repository
        .delete(notification_id)
        .await?;
    
    Ok(json_deleted())
}

/// Ejecutar cleanup de notificaciones antiguas (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn cleanup_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<CleanupParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ejecutar cleanup de notificaciones".to_string()
        ));
    }
    
    info!("[DELETE] Ejecutando cleanup de notificaciones");
    
    let result = state.container.notification_repository
        .cleanup_by_priority(
            params.days_low,
            params.days_normal,
            params.days_high,
            params.days_urgent,
        )
        .await?;
    
    info!(
        "Cleanup completado: {} expiradas, {} low, {} normal, {} high, {} urgent = {} total",
        result.deleted_expired,
        result.deleted_low,
        result.deleted_normal,
        result.deleted_high,
        result.deleted_urgent,
        result.total_deleted
    );
    
    Ok(json_ok(serde_json::json!({
        "success": true,
        "deleted_expired": result.deleted_expired,
        "deleted_low": result.deleted_low,
        "deleted_normal": result.deleted_normal,
        "deleted_high": result.deleted_high,
        "deleted_urgent": result.deleted_urgent,
        "total_deleted": result.total_deleted,
        "config": {
            "days_low": params.days_low,
            "days_normal": params.days_normal,
            "days_high": params.days_high,
            "days_urgent": params.days_urgent
        }
    })))
}
