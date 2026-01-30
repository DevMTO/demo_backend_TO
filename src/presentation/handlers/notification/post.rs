//! POST handlers para Notification

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};
use validator::Validate;

use crate::domain::entities::{
    NotificationType, NotificationCategory, NotificationPriority, UserRole,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_created;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::CreateNotificationRequest;

/// Crear una nueva notificación (solo SuperAdmin/Admin)
#[instrument(skip(state, auth, request))]
pub async fn create_notification(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateNotificationRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !auth.user.role.is_super_admin() && !auth.user.role.is_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden crear notificaciones".to_string()
        ));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    info!("📢 Creando notificación: {}", request.title);
    
    let notification_type = request.notification_type
        .parse::<NotificationType>()
        .map_err(|_| ApplicationError::Validation("Tipo de notificación inválido".to_string()))?;
    
    let category = request.category
        .parse::<NotificationCategory>()
        .map_err(|_| ApplicationError::Validation("Categoría inválida".to_string()))?;
    
    let priority = request.priority
        .parse::<NotificationPriority>()
        .map_err(|_| ApplicationError::Validation("Prioridad inválida".to_string()))?;
    
    let target_roles: Option<Vec<UserRole>> = if let Some(ref roles) = request.target_roles {
        Some(
            roles.iter()
                .map(|r| r.parse::<UserRole>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| ApplicationError::Validation("Rol inválido en target_roles".to_string()))?
        )
    } else {
        None
    };
    
    let notification = if let Some(target_user_id) = request.target_user_id {
        state.container.notification_service
            .notify_user(
                target_user_id,
                &request.title,
                &request.message,
                notification_type,
                category,
                priority,
                Some(auth.user.id),
            )
            .await?
    } else if let Some(roles) = target_roles {
        state.container.notification_service
            .notify_roles(
                roles,
                &request.title,
                &request.message,
                notification_type,
                category,
                priority,
                Some(auth.user.id),
            )
            .await?
    } else {
        state.container.notification_service
            .notify_all(
                &request.title,
                &request.message,
                notification_type,
                category,
                priority,
                Some(auth.user.id),
            )
            .await?
    };
    
    info!("Notificación creada con ID: {}", notification.id);
    
    Ok(json_created(serde_json::json!({
        "id": notification.id,
        "title": notification.title,
        "message": notification.message,
        "created_at": notification.created_at.to_rfc3339(),
    })))
}
