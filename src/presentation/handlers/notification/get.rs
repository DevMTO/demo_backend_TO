//! GET handlers para Notification

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{UserNotificationDto, UserNotificationListDto, UnreadCountDto};

use super::query_params::ListNotificationsParams;

/// Obtener notificaciones del usuario autenticado
#[instrument(skip(state, auth))]
pub async fn get_my_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListNotificationsParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Obteniendo notificaciones para usuario: {}", auth.user.id);
    
    let page_size = params.page_size.clamp(1, 10000); // Aumentado de 100 a 10000
    let offset = (params.page - 1).max(0) * page_size;
    
    let notifications = state.container.notification_service
        .get_user_notifications(
            auth.user.id,
            params.unread_only.unwrap_or(false),
            page_size,
            offset,
        )
        .await?;
    
    let total = state.container.notification_repository
        .count_user_notifications(auth.user.id, params.unread_only.unwrap_or(false))
        .await?;
    
    let dto_list: Vec<UserNotificationDto> = notifications
        .into_iter()
        .map(UserNotificationDto::from)
        .collect();
    
    let response = UserNotificationListDto {
        notifications: dto_list,
        total,
        page: params.page,
        page_size,
        total_pages: (total as f64 / page_size as f64).ceil() as i64,
    };
    
    Ok(json_ok(response))
}

/// Obtener contador de notificaciones no leídas
#[instrument(skip(state, auth))]
pub async fn get_unread_count(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🔢 Obteniendo contador de no leídas para usuario: {}", auth.user.id);
    
    let count = state.container.notification_service
        .get_unread_count(auth.user.id)
        .await?;
    
    Ok(json_ok(UnreadCountDto {
        unread_count: count,
        user_id: auth.user.id,
    }))
}

/// Listar todas las notificaciones del sistema (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn list_all_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListNotificationsParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ver todas las notificaciones".to_string()
        ));
    }
    
    info!("Listando todas las notificaciones del sistema");
    
    let page_size = params.page_size.clamp(1, 10000); // Aumentado de 100 a 10000
    let offset = (params.page - 1).max(0) * page_size;
    
    let filters = crate::application::ports::NotificationFilters {
        notification_type: params.notification_type.and_then(|s| s.parse().ok()),
        category: params.category.and_then(|s| s.parse().ok()),
        ..Default::default()
    };
    
    let notifications = state.container.notification_repository
        .find_all(filters.clone(), page_size, offset)
        .await?;
    
    let total = state.container.notification_repository
        .count(filters)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "notifications": notifications,
        "total": total,
        "page": params.page,
        "page_size": page_size,
        "total_pages": (total as f64 / page_size as f64).ceil() as i64
    })))
}
