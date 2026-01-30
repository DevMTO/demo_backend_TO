//! Rutas de notificaciones

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::notification;

use super::state::AppState;

pub fn notification_routes() -> Router<AppState> {
    Router::new()
        // SSE para notificaciones en tiempo real
        .route("/sse", get(notification::notifications_sse))
        // User notifications (mis notificaciones)
        .route("/me", get(notification::get_my_notifications))
        .route("/me/unread-count", get(notification::get_unread_count))
        .route("/me/read-all", post(notification::mark_all_as_read))
        .route("/me/dismiss-all", post(notification::dismiss_all_notifications))
        .route("/me/{id}/read", post(notification::mark_as_read))
        .route("/me/{id}/dismiss", post(notification::dismiss_notification))
        // Admin notifications (crear, listar todas, eliminar)
        .route("/", get(notification::list_all_notifications).post(notification::create_notification))
        .route("/cleanup", post(notification::cleanup_notifications))
        .route("/{id}", axum::routing::delete(notification::delete_notification))
}
