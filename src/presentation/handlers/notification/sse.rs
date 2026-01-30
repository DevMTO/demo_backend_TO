//! SSE handlers para Notification

use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tracing::{info, instrument, warn};

use crate::infrastructure::sse::SseEvent;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

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
    
    let mut receiver = state.broadcaster.subscribe(user_id).await;
    
    let stream = async_stream::stream! {
        let connected_event = SseEvent::Connected { user_id };
        if let Ok(json) = serde_json::to_string(&connected_event) {
            yield Ok(Event::default()
                .event("connected")
                .data(json));
        }
        
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
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
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("SSE: Canal cerrado para usuario {}", user_id);
                            break;
                        }
                    }
                }
                
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
