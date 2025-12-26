//! # HTTP Routes
//! 
//! Definición de rutas de la API con cookies de sesión.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    routing::{get, post},
    http::{header, Method, HeaderValue},
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::compression::CompressionLayer;
use tower_cookies::CookieManagerLayer;

use crate::config::AppConfig;
use crate::infrastructure::container::DependencyContainer;
use super::handlers::{
    login_handler,
    logout_handler,
    verify_session_handler,
    health_check,
};

/// Estado compartido de la aplicación
#[derive(Clone)]
pub struct AppState {
    pub container: Arc<DependencyContainer>,
}

/// Crear el router principal de la aplicación
pub fn create_router(container: Arc<DependencyContainer>, config: &AppConfig) -> Router {
    let state = AppState { container };
    
    // Configurar CORS
    let cors = create_cors_layer(config);
    
    // Router de autenticación
    let auth_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify", get(verify_session_handler))
        .route("/me", get(verify_session_handler)); // Alias para verificar sesión actual
    
    // Router de health check
    let health_routes = Router::new()
        .route("/", get(health_check));
    
    // Router principal
    Router::new()
        .nest("/api/v1/auth", auth_routes)
        .nest("/health", health_routes)
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB max
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30)
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Crear capa de CORS segura
fn create_cors_layer(config: &AppConfig) -> CorsLayer {
    let origins: Vec<HeaderValue> = config.cors_allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(config.cors_max_age_secs))
}
