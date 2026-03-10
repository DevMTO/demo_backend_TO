//! Rutas de la API
//! 
//! Este módulo organiza todas las rutas de la API en submódulos separados por entidad.

mod state;
mod auth;
mod health;
mod persona;
mod agencia;
mod tour;
mod transporte;
mod vehiculo;
mod conductor;
mod guia;
mod restaurante;
mod entrada;
mod entrada_precio;
mod file;
mod file_tour;
mod file_vehiculos;
mod user;
mod activity_log;
mod notification;
mod storage;
mod contabilidad;
mod my_files;
mod mis_pagos;
mod cadena_hotelera;
mod hotel;
mod tarifa;

// Re-exportar AppState para uso externo
pub use state::AppState;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
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
use super::middleware::require_auth;

pub fn create_router(
    container: Arc<DependencyContainer>, 
    broadcaster: Arc<NotificationBroadcaster>,
    config: &AppConfig
) -> Router {
    let state = AppState::new(container, broadcaster);
    
    // Configurar CORS
    let cors = create_cors_layer(config);
    
    // ========== RUTAS PROTEGIDAS ==========
    // Todas las rutas CRUD requieren autenticación vía cookie de sesión
    let protected_routes = Router::new()
        .nest("/users", user::user_routes())
        .nest("/personas", persona::persona_routes())
        .nest("/agencias", agencia::agencia_routes())
        .nest("/tours", tour::tour_routes())
        .nest("/transportes", transporte::transporte_routes())
        .nest("/vehiculos", vehiculo::vehiculo_routes())
        .nest("/conductores", conductor::conductor_routes())
        .nest("/guias", guia::guia_routes())
        .nest("/restaurantes", restaurante::restaurante_routes())
        .nest("/entradas", entrada::entrada_routes())
        .nest("/entrada-precios", entrada_precio::entrada_precio_routes())
        .nest("/files", file::file_routes())
        .nest("/file-tours", file_tour::file_tour_routes())
        .nest("/file-vehiculos", file_vehiculos::file_vehiculos_routes())
        .nest("/logs", activity_log::activity_log_routes())
        .nest("/notifications", notification::notification_routes())
        .nest("/storage", storage::storage_routes())
        .nest("/my-files", my_files::my_files_routes())
        .nest("/contabilidad", contabilidad::contabilidad_routes())
        .nest("/mis-pagos", mis_pagos::mis_pagos_routes())
        .nest("/cadenas-hoteleras", cadena_hotelera::cadena_hotelera_routes())
        .nest("/hoteles", hotel::hotel_routes())
        .nest("/tarifas", tarifa::tarifa_routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Router principal con capas de seguridad
    let router = Router::new()
        .nest("/api/v1/auth", auth::auth_routes())
        .nest("/api/v1", protected_routes)
        .nest("/health", health::health_routes())
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
