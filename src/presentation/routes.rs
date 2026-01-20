use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    routing::{get, post, patch},
    middleware,
};
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::compression::CompressionLayer;
use tower_cookies::CookieManagerLayer;

use crate::infrastructure::security::http_security::{create_cors_layer, apply_security_layers, method_filter};

use crate::config::AppConfig;
use crate::infrastructure::container::DependencyContainer;
use crate::infrastructure::sse::NotificationBroadcaster;
use super::handlers::{
    login_handler,
    logout_handler,
    verify_session_handler,
    health_check,
    get_profile_handler,
    update_profile_handler,
    user_handlers,
    persona_handlers,
    agencia_handlers,
    tour_handlers,
    transporte_handlers,
    vehiculo_handlers,
    conductor_handlers,
    guia_handlers,
    restaurante_handlers,
    entrada_handlers,
    file_handlers,
    file_relations_handlers,
    my_files_handlers,
    pago_handlers,
    activity_log_handlers,
    notification_handlers,
    storage_handlers,
};
use super::middleware::require_auth;

use crate::application::dtos::UserNotificationDto;
use crate::domain::entities::{
    NotificationType, NotificationCategory, NotificationPriority, UserRole,
};
use crate::infrastructure::sse::SseEvent;

#[derive(Clone)]
pub struct AppState {
    pub container: Arc<DependencyContainer>,
    pub broadcaster: Arc<NotificationBroadcaster>,
}

impl AppState {
    /// Notificar a roles específicos y enviar por SSE
    pub async fn notify_roles_with_broadcast(
        &self,
        roles: Vec<UserRole>,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        category: NotificationCategory,
        priority: NotificationPriority,
        created_by: Option<i32>,
    ) -> Result<(), crate::domain::errors::ApplicationError> {
        // 1. Crear notificación en DB
        let notification = self.container.notification_service.notify_roles(
            roles.clone(),
            title,
            message,
            notification_type.clone(),
            category.clone(),
            priority.clone(),
            created_by,
        ).await?;

        // 2. Obtener IDs de usuarios con esos roles
        let roles_str: Vec<String> = roles.iter().map(|r| r.to_string().to_lowercase()).collect();
        let user_ids = self.container.notification_repository.get_users_by_roles(roles_str).await?;

        // 3. Enviar por SSE a cada usuario conectado
        let dto = UserNotificationDto {
            id: notification.id,
            title: notification.title,
            message: notification.message,
            notification_type: notification.notification_type.to_string(),
            category: notification.category.to_string(),
            priority: notification.priority.to_string(),
            entity_type: notification.entity_type.clone(),
            entity_id: notification.entity_id,
            metadata: notification.metadata.clone(),
            is_read: false,
            read_at: None,
            is_dismissed: false,
            created_at: notification.created_at,
        };

        let event = SseEvent::NewNotification(dto);
        for user_id in user_ids {
            // Excluir al usuario que creó la notificación del broadcast SSE
            if Some(user_id) != created_by {
                self.broadcaster.send_to_user(user_id, event.clone()).await;
            }
        }

        Ok(())
    }
}

pub fn create_router(
    container: Arc<DependencyContainer>, 
    broadcaster: Arc<NotificationBroadcaster>,
    config: &AppConfig
) -> Router {
    let state = AppState { container, broadcaster };
    
    // Configurar CORS
    let cors = create_cors_layer(config);
    
    // Router de autenticación
    let auth_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify", get(verify_session_handler))
        .route("/me", get(verify_session_handler))
        .route("/profile", get(get_profile_handler).put(update_profile_handler));
    
    // Router de health check
    let health_routes = Router::new()
        .route("/", get(health_check));

    // === CRUD Routes ===
    
    let persona_routes = Router::new()
        .route("/", get(persona_handlers::list_personas).post(persona_handlers::create_persona))
        .route("/search", get(persona_handlers::search_personas))
        .route("/{id}", get(persona_handlers::get_persona).put(persona_handlers::update_persona).delete(persona_handlers::delete_persona));

    let agencia_routes = Router::new()
        .route("/", get(agencia_handlers::list_agencias).post(agencia_handlers::create_agencia))
        .route("/mi-agencia", get(agencia_handlers::get_mi_agencia).put(agencia_handlers::update_mi_agencia))
        .route("/mi-agencia/interfaz", patch(agencia_handlers::patch_mi_agencia_interfaz))
        .route("/ruc/{ruc}", get(agencia_handlers::get_agencia_by_ruc))
        .route("/{id}", get(agencia_handlers::get_agencia).put(agencia_handlers::update_agencia).delete(agencia_handlers::delete_agencia))
        .route("/{id}/restore", patch(agencia_handlers::restore_agencia));

    let tour_routes = Router::new()
        .route("/", get(tour_handlers::list_tours).post(tour_handlers::create_tour))
        .route("/search", get(tour_handlers::search_tours))
        .route("/{id}", get(tour_handlers::get_tour).put(tour_handlers::update_tour).delete(tour_handlers::delete_tour))
        .route("/{id}/restore", patch(tour_handlers::restore_tour));

    let transporte_routes = Router::new()
        .route("/", get(transporte_handlers::list_transportes).post(transporte_handlers::create_transporte))
        .route("/mi-transporte", get(transporte_handlers::get_mi_transporte).put(transporte_handlers::update_mi_transporte))
        .route("/mi-transporte/interfaz", patch(transporte_handlers::patch_mi_transporte_interfaz))
        .route("/with-vehicles", get(transporte_handlers::list_transportes_with_vehicles))
        .route("/{id}", get(transporte_handlers::get_transporte).put(transporte_handlers::update_transporte).delete(transporte_handlers::delete_transporte))
        .route("/{id}/restore", patch(transporte_handlers::restore_transporte));

    let vehiculo_routes = Router::new()
        .route("/", get(vehiculo_handlers::list_vehiculos).post(vehiculo_handlers::create_vehiculo))
        .route("/available", get(vehiculo_handlers::list_vehiculos_available))
        .route("/transporte/{transporte_id}", get(vehiculo_handlers::list_vehiculos_by_transporte))
        .route("/{id}", get(vehiculo_handlers::get_vehiculo).put(vehiculo_handlers::update_vehiculo).delete(vehiculo_handlers::delete_vehiculo));

    let conductor_routes = Router::new()
        .route("/", get(conductor_handlers::list_conductores).post(conductor_handlers::create_conductor))
        .route("/available", get(conductor_handlers::list_conductores_available))
        .route("/transporte/{transporte_id}", get(conductor_handlers::list_conductores_by_transporte))
        .route("/{id}", get(conductor_handlers::get_conductor).put(conductor_handlers::update_conductor).delete(conductor_handlers::delete_conductor));

    let guia_routes = Router::new()
        .route("/", get(guia_handlers::list_guias).post(guia_handlers::create_guia))
        .route("/search", get(guia_handlers::search_guias))
        .route("/available", get(guia_handlers::list_guias_available))
        .route("/{id}", get(guia_handlers::get_guia).put(guia_handlers::update_guia).delete(guia_handlers::delete_guia));

    let restaurante_routes = Router::new()
        .route("/", get(restaurante_handlers::list_restaurantes).post(restaurante_handlers::create_restaurante))
        .route("/search", get(restaurante_handlers::search_restaurantes))
        .route("/{id}", get(restaurante_handlers::get_restaurante).put(restaurante_handlers::update_restaurante).delete(restaurante_handlers::delete_restaurante))
        .route("/{id}/restore", patch(restaurante_handlers::restore_restaurante));

    let entrada_routes = Router::new()
        .route("/", get(entrada_handlers::list_entradas).post(entrada_handlers::create_entrada))
        .route("/search", get(entrada_handlers::search_entradas))
        .route("/{id}", get(entrada_handlers::get_entrada).put(entrada_handlers::update_entrada).delete(entrada_handlers::delete_entrada))
        .route("/{id}/restore", patch(entrada_handlers::restore_entrada));

    let file_routes = Router::new()
        .route("/", get(file_handlers::list_files).post(file_handlers::create_file))
        .route("/upcoming", get(file_handlers::list_files_upcoming))
        .route("/pending-payment", get(file_handlers::list_files_pending_payment))
        .route("/by-date", get(file_handlers::list_files_by_date_range))
        .route("/agencia/{agencia_id}", get(file_handlers::list_files_by_agencia))
        .route("/{id}", get(file_handlers::get_file).put(file_handlers::update_file).delete(file_handlers::delete_file))
        // File Relations - Guías
        .route("/{id}/guias", get(file_relations_handlers::list_file_guias).post(file_relations_handlers::assign_guia_to_file))
        .route("/{id}/guias/{guia_id}", axum::routing::delete(file_relations_handlers::remove_file_guia))
        // File Relations - Pasajeros
        .route("/{id}/pasajeros", get(file_relations_handlers::list_file_pasajeros).post(file_relations_handlers::add_pasajero_to_file))
        .route("/{id}/pasajeros/with-persona", post(file_relations_handlers::create_pasajero_with_persona))
        .route("/{id}/pasajeros/{pasajero_id}", axum::routing::delete(file_relations_handlers::remove_file_pasajero))
        // File Relations - Vehículos
        .route("/{id}/vehiculos", get(file_relations_handlers::list_file_vehiculos).post(file_relations_handlers::assign_vehiculo_to_file))
        .route("/{id}/vehiculos/{vehiculo_id}", axum::routing::delete(file_relations_handlers::remove_file_vehiculo))
        .route("/{id}/vehiculos/{vehiculo_id}/status", axum::routing::put(file_relations_handlers::update_vehiculo_status));
    
    // File Tour relations (entradas y restaurantes ahora vinculados a file_tours)
    let file_tour_relation_routes = Router::new()
        .route("/{file_tour_id}/entradas", get(file_relations_handlers::list_file_tour_entradas))
        .route("/{file_tour_id}/restaurantes", get(file_relations_handlers::list_file_tour_restaurantes))
        .route("/entradas", post(file_relations_handlers::assign_entrada_to_file_tour))
        .route("/entradas/{entrada_asig_id}", axum::routing::delete(file_relations_handlers::remove_file_entrada))
        .route("/restaurantes", post(file_relations_handlers::assign_restaurante_to_file_tour))
        .route("/restaurantes/{restaurante_asig_id}", axum::routing::delete(file_relations_handlers::remove_file_restaurante));

    let pago_routes = Router::new()
        .route("/", get(pago_handlers::list_pagos).post(pago_handlers::create_pago))
        .route("/file/{file_id}", get(pago_handlers::list_pagos_by_file))
        .route("/file/{file_id}/balance", get(pago_handlers::get_file_balance))
        .route("/{id}", get(pago_handlers::get_pago).put(pago_handlers::update_pago).delete(pago_handlers::delete_pago));

    let user_routes = Router::new()
        .route("/", get(user_handlers::list_users).post(user_handlers::create_user))
        .route("/{id}", get(user_handlers::get_user).put(user_handlers::update_user).delete(user_handlers::delete_user))
        .route("/{id}/activate", patch(user_handlers::activate_user))
        .route("/{id}/deactivate", patch(user_handlers::deactivate_user))
        .route("/{id}/change-password", patch(user_handlers::admin_change_password));

    // === System Routes (Logs & Notifications) ===
    
    let activity_log_routes = Router::new()
        .route("/", get(activity_log_handlers::list_logs))
        .route("/summary", get(activity_log_handlers::get_logs_summary))
        .route("/errors", get(activity_log_handlers::get_recent_errors))
        .route("/cleanup", post(activity_log_handlers::cleanup_old_logs));
    
    let notification_routes = Router::new()
        // SSE para notificaciones en tiempo real
        .route("/sse", get(notification_handlers::notifications_sse))
        // User notifications (mis notificaciones)
        .route("/me", get(notification_handlers::get_my_notifications))
        .route("/me/unread-count", get(notification_handlers::get_unread_count))
        .route("/me/read-all", post(notification_handlers::mark_all_as_read))
        .route("/me/dismiss-all", post(notification_handlers::dismiss_all_notifications))
        .route("/me/{id}/read", post(notification_handlers::mark_as_read))
        .route("/me/{id}/dismiss", post(notification_handlers::dismiss_notification))
        // Admin notifications (crear, listar todas, eliminar)
        .route("/", get(notification_handlers::list_all_notifications).post(notification_handlers::create_notification))
        .route("/cleanup", post(notification_handlers::cleanup_notifications))
        .route("/{id}", axum::routing::delete(notification_handlers::delete_notification));

    // === Storage Routes (Tigris) ===
    // Rutas de subida, eliminación y proxy (todas protegidas)
    let storage_routes = Router::new()
        .route("/agencia/{agencia_id}/logo", post(storage_handlers::upload_agencia_logo).delete(storage_handlers::delete_agencia_logo))
        .route("/agencia/{agencia_id}/banner", post(storage_handlers::upload_agencia_banner).delete(storage_handlers::delete_agencia_banner))
        .route("/transporte/{transporte_id}/logo", post(storage_handlers::upload_transporte_logo))
        .route("/tour/{tour_id}/image", post(storage_handlers::upload_tour_image))
        .route("/proxy/{*file_path}", get(storage_handlers::proxy_file));

    // === My Files Routes (para guías, conductores, restaurantes) ===
    // Cada usuario ve solo los files donde está asignado
    let my_files_routes = Router::new()
        .route("/guia", get(my_files_handlers::get_my_files_as_guia))
        .route("/guia/{file_id}/confirm", post(my_files_handlers::confirm_guia_assignment))
        .route("/conductor", get(my_files_handlers::get_my_files_as_conductor))
        .route("/conductor/{file_id}/confirm", post(my_files_handlers::confirm_conductor_assignment))
        .route("/restaurante", get(my_files_handlers::get_my_files_as_restaurante));

    // === File Vehiculos Routes (listar todas las asignaciones vehículo-file) ===
    let file_vehiculos_routes = Router::new()
        .route("/", get(file_relations_handlers::list_all_file_vehiculos));

    // ========== RUTAS PROTEGIDAS ==========
    // Todas las rutas CRUD requieren autenticación vía cookie de sesión
    let protected_routes = Router::new()
        .nest("/users", user_routes)
        .nest("/personas", persona_routes)
        .nest("/agencias", agencia_routes)
        .nest("/tours", tour_routes)
        .nest("/transportes", transporte_routes)
        .nest("/vehiculos", vehiculo_routes)
        .nest("/conductores", conductor_routes)
        .nest("/guias", guia_routes)
        .nest("/restaurantes", restaurante_routes)
        .nest("/entradas", entrada_routes)
        .nest("/files", file_routes)
        .nest("/file-tours", file_tour_relation_routes)
        .nest("/file-vehiculos", file_vehiculos_routes)
        .nest("/pagos", pago_routes)
        .nest("/logs", activity_log_routes)
        .nest("/notifications", notification_routes)
        .nest("/storage", storage_routes)
        .nest("/my-files", my_files_routes)
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Router principal con capas de seguridad
    let router = Router::new()
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1", protected_routes)
        .nest("/health", health_routes)
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB max
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30)
        ))
        // Middleware para filtrar métodos HTTP no permitidos
        .layer(middleware::from_fn(method_filter))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);
    
    // Aplicar capas de seguridad HTTP (headers, request ID)
    apply_security_layers(router)
}
