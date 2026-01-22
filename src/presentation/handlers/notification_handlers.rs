use axum::{
    extract::{Path, Query, State},
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
    Json,
};
use futures::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::time::Duration;
use tracing::{info, instrument, warn};
use validator::Validate;

use crate::domain::entities::{
    NotificationType, 
    NotificationCategory, 
    NotificationPriority,
    UserRole,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::sse::SseEvent;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{
    CreateNotificationRequest, 
    UserNotificationDto, 
    UserNotificationListDto,
    UnreadCountDto,
};

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

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

/// Obtener notificaciones del usuario autenticado
#[instrument(skip(state, auth))]
pub async fn get_my_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListNotificationsParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Obteniendo notificaciones para usuario: {}", auth.user.id);
    
    let page_size = params.page_size.min(100).max(1);
    let offset = (params.page - 1).max(0) * page_size;
    
    let notifications = state.container.notification_service
        .get_user_notifications(
            auth.user.id,
            params.unread_only.unwrap_or(false),
            page_size,
            offset,
        )
        .await?;
    
    // Obtener total
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

// ============== Admin endpoints (crear notificaciones manualmente) ==============

/// Crear una nueva notificación (solo SuperAdmin/Admin)
#[instrument(skip(state, auth, request))]
pub async fn create_notification(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateNotificationRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar permisos (SuperAdmin o Admin)
    if !auth.user.role.is_super_admin() && !auth.user.role.is_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden crear notificaciones".to_string()
        ));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    info!("📢 Creando notificación: {}", request.title);
    
    // Parsear enums
    let notification_type = request.notification_type
        .parse::<NotificationType>()
        .map_err(|_| ApplicationError::Validation("Tipo de notificación inválido".to_string()))?;
    
    let category = request.category
        .parse::<NotificationCategory>()
        .map_err(|_| ApplicationError::Validation("Categoría inválida".to_string()))?;
    
    let priority = request.priority
        .parse::<NotificationPriority>()
        .map_err(|_| ApplicationError::Validation("Prioridad inválida".to_string()))?;
    
    // Parsear target_roles si existe
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
    
    // Crear según el tipo de destinatario
    let notification = if let Some(target_user_id) = request.target_user_id {
        // Notificación para un usuario específico
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
        // Notificación para roles específicos
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
        // Notificación para todos
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

/// Listar todas las notificaciones del sistema (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn list_all_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListNotificationsParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ver todas las notificaciones".to_string()
        ));
    }
    
    info!("Listando todas las notificaciones del sistema");
    
    let page_size = params.page_size.min(100).max(1);
    let offset = (params.page - 1).max(0) * page_size;
    
    // Construir filtros
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

/// Eliminar una notificación (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn delete_notification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(notification_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede eliminar notificaciones".to_string()
        ));
    }
    
    info!("🗑️ Eliminando notificación: {}", notification_id);
    
    // Verificar que existe
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

fn default_days_low() -> i32 { 7 }
fn default_days_normal() -> i32 { 14 }
fn default_days_high() -> i32 { 30 }
fn default_days_urgent() -> i32 { 60 }

/// Ejecutar cleanup de notificaciones antiguas (solo SuperAdmin)
#[instrument(skip(state, auth))]
pub async fn cleanup_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<CleanupParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ejecutar cleanup de notificaciones".to_string()
        ));
    }
    
    info!("🗑️ Ejecutando cleanup de notificaciones");
    
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

// ============== SSE Endpoint para notificaciones en tiempo real ==============

/// Handler SSE para notificaciones en tiempo real
/// 
/// Este endpoint establece una conexión Server-Sent Events que permite
/// recibir notificaciones en tiempo real sin polling.
/// 
/// Eventos emitidos:
/// - `new_notification`: Nueva notificación recibida
/// - `notification_read`: Notificación marcada como leída  
/// - `unread_count`: Contador de no leídas actualizado
/// - `heartbeat`: Keepalive cada 30 segundos
/// - `connected`: Confirmación de conexión establecida
#[instrument(skip(state, auth))]
pub async fn notifications_sse(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = auth.user.id;
    info!("📡 SSE: Usuario {} conectado para recibir notificaciones", user_id);
    
    // Suscribirse al broadcaster
    let mut receiver = state.broadcaster.subscribe(user_id).await;
    
    // Crear stream de eventos
    let stream = async_stream::stream! {
        // Enviar evento de conexión establecida
        let connected_event = SseEvent::Connected { user_id };
        if let Ok(json) = serde_json::to_string(&connected_event) {
            yield Ok(Event::default()
                .event("connected")
                .data(json));
        }
        
        // Intervalo para heartbeat (cada 30 segundos)
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // Recibir eventos del broadcaster
                result = receiver.recv() => {
                    match result {
                        Ok(event) => {
                            let (event_type, json) = match &event {
                                SseEvent::NewNotification(notification) => {
                                    ("new_notification", serde_json::to_string(&notification))
                                }
                                SseEvent::NotificationRead { notification_id } => {
                                    ("notification_read", serde_json::to_string(&serde_json::json!({
                                        "notification_id": notification_id
                                    })))
                                }
                                SseEvent::NotificationDismissed { notification_id } => {
                                    ("notification_dismissed", serde_json::to_string(&serde_json::json!({
                                        "notification_id": notification_id
                                    })))
                                }
                                SseEvent::AllNotificationsRead => {
                                    ("all_read", serde_json::to_string(&serde_json::json!({
                                        "success": true
                                    })))
                                }
                                SseEvent::AllNotificationsDismissed => {
                                    ("all_dismissed", serde_json::to_string(&serde_json::json!({
                                        "success": true
                                    })))
                                }
                                SseEvent::UnreadCountUpdated { count } => {
                                    ("unread_count", serde_json::to_string(&serde_json::json!({
                                        "count": count
                                    })))
                                }
                                SseEvent::Heartbeat => {
                                    ("heartbeat", Ok("{}".to_string()))
                                }
                                SseEvent::Connected { user_id } => {
                                    ("connected", serde_json::to_string(&serde_json::json!({
                                        "user_id": user_id
                                    })))
                                }
                            };
                            
                            if let Ok(data) = json {
                                yield Ok(Event::default()
                                    .event(event_type)
                                    .data(data));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("SSE: Usuario {} perdió {} mensajes", user_id, n);
                            // Continuar recibiendo
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("SSE: Canal cerrado para usuario {}", user_id);
                            break;
                        }
                    }
                }
                
                // Enviar heartbeat periódico
                _ = heartbeat_interval.tick() => {
                    yield Ok(Event::default()
                        .event("heartbeat")
                        .data("{}"));
                }
            }
        }
        
        info!("📡 SSE: Usuario {} desconectado", user_id);
    };
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive")
    )
}
