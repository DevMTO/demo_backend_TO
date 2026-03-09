use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use crate::application::dtos::UserNotificationDto;

/// Capacidad del canal broadcast
const BROADCAST_CAPACITY: usize = 256;

/// Evento SSE para notificaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SseEvent {
    /// Nueva notificación recibida
    NewNotification(Box<UserNotificationDto>),
    /// Notificación marcada como leída
    NotificationRead { notification_id: i32 },
    /// Notificación descartada
    NotificationDismissed { notification_id: i32 },
    /// Todas las notificaciones marcadas como leídas
    AllNotificationsRead,
    /// Todas las notificaciones descartadas
    AllNotificationsDismissed,
    /// Contador de no leídas actualizado
    UnreadCountUpdated { count: i64 },
    /// Heartbeat para mantener la conexión viva
    Heartbeat,
    /// Conexión establecida
    Connected { user_id: i32 },
}

/// Mensaje interno del broadcaster
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    /// ID del usuario destinatario (None = broadcast a todos)
    pub target_user_id: Option<i32>,
    /// Roles destinatarios (None = todos los roles)
    pub target_roles: Option<Vec<String>>,
    /// Evento a enviar
    pub event: SseEvent,
}

/// Broadcaster SSE para notificaciones
/// 
/// Mantiene canales de broadcast por usuario y permite enviar
/// eventos tanto a usuarios específicos como a todos los conectados.
#[derive(Clone)]
pub struct NotificationBroadcaster {
    /// Canal broadcast global (para notificaciones a todos)
    #[allow(dead_code)]
    global_sender: broadcast::Sender<BroadcastMessage>,
    /// Canales por usuario
    user_channels: Arc<RwLock<HashMap<i32, broadcast::Sender<SseEvent>>>>,
}

impl NotificationBroadcaster {
    /// Crear nuevo broadcaster
    pub fn new() -> Self {
        let (global_sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            global_sender,
            user_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Suscribir un usuario y obtener su receiver
    pub async fn subscribe(&self, user_id: i32) -> broadcast::Receiver<SseEvent> {
        let mut channels = self.user_channels.write().await;
        
        // Si ya existe un canal para este usuario, retornar nuevo subscriber
        if let Some(sender) = channels.get(&user_id) {
            return sender.subscribe();
        }
        
        // Crear nuevo canal para el usuario
        let (sender, receiver) = broadcast::channel(BROADCAST_CAPACITY);
        channels.insert(user_id, sender);
        
        receiver
    }
    
    /// Obtener subscriber del canal global
    #[allow(dead_code)]
    pub fn subscribe_global(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.global_sender.subscribe()
    }
    
    /// Desuscribir usuario (limpieza cuando se desconecta)
    #[allow(dead_code)]
    pub async fn unsubscribe(&self, user_id: i32) {
        let mut channels = self.user_channels.write().await;
        channels.remove(&user_id);
    }
    
    /// Enviar evento a un usuario específico
    #[allow(dead_code)]
    pub async fn send_to_user(&self, user_id: i32, event: SseEvent) -> bool {
        let channels = self.user_channels.read().await;
        if let Some(sender) = channels.get(&user_id) {
            let result = sender.send(event.clone());
            if result.is_ok() {
                tracing::info!("📤 SSE: Evento enviado a usuario {}", user_id);
                true
            } else {
                tracing::warn!("📤 SSE: Error enviando evento a usuario {} (canal cerrado)", user_id);
                false
            }
        } else {
            tracing::debug!("📤 SSE: Usuario {} no tiene canal SSE activo", user_id);
            false
        }
    }
    
    /// Enviar evento a todos los usuarios conectados
    #[allow(dead_code)]
    pub async fn broadcast(&self, event: SseEvent) {
        let channels = self.user_channels.read().await;
        for sender in channels.values() {
            let _ = sender.send(event.clone());
        }
    }
    
    /// Enviar evento a usuarios con roles específicos
    /// Nota: Requiere que el frontend mantenga la información del rol
    #[allow(dead_code)]
    pub async fn send_to_roles(&self, roles: Vec<String>, event: SseEvent) {
        // Enviar mensaje global con filtro de roles
        let message = BroadcastMessage {
            target_user_id: None,
            target_roles: Some(roles),
            event,
        };
        let _ = self.global_sender.send(message);
    }
    
    /// Enviar nueva notificación
    #[allow(dead_code)]
    pub async fn notify_new(&self, user_id: Option<i32>, notification: UserNotificationDto) {
        let event = SseEvent::NewNotification(Box::new(notification));
        
        if let Some(uid) = user_id {
            self.send_to_user(uid, event).await;
        } else {
            self.broadcast(event).await;
        }
    }
    
    /// Notificar que una notificación fue leída
    #[allow(dead_code)]
    pub async fn notify_read(&self, user_id: i32, notification_id: i32) {
        let event = SseEvent::NotificationRead { notification_id };
        self.send_to_user(user_id, event).await;
    }
    
    /// Notificar actualización del contador de no leídas
    #[allow(dead_code)]
    pub async fn notify_unread_count(&self, user_id: i32, count: i64) {
        let event = SseEvent::UnreadCountUpdated { count };
        self.send_to_user(user_id, event).await;
    }
    
    /// Obtener número de usuarios conectados
    #[allow(dead_code)]
    pub async fn connected_count(&self) -> usize {
        self.user_channels.read().await.len()
    }
    
    /// Verificar si un usuario está conectado
    #[allow(dead_code)]
    pub async fn is_connected(&self, user_id: i32) -> bool {
        self.user_channels.read().await.contains_key(&user_id)
    }
}

impl Default for NotificationBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
