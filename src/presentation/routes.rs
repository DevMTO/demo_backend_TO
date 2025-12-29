use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    routing::{get, post},
    http::{header, Method, HeaderValue},
    middleware,
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
    pago_handlers,
};
use super::middleware::require_auth;

#[derive(Clone)]
pub struct AppState {
    pub container: Arc<DependencyContainer>,
}

pub fn create_router(container: Arc<DependencyContainer>, config: &AppConfig) -> Router {
    let state = AppState { container };
    
    // Configurar CORS
    let cors = create_cors_layer(config);
    
    // Router de autenticación
    let auth_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify", get(verify_session_handler))
        .route("/me", get(verify_session_handler));
    
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
        .route("/ruc/{ruc}", get(agencia_handlers::get_agencia_by_ruc))
        .route("/{id}", get(agencia_handlers::get_agencia).put(agencia_handlers::update_agencia).delete(agencia_handlers::delete_agencia))
        .route("/{id}/restore", post(agencia_handlers::restore_agencia));

    let tour_routes = Router::new()
        .route("/", get(tour_handlers::list_tours).post(tour_handlers::create_tour))
        .route("/search", get(tour_handlers::search_tours))
        .route("/{id}", get(tour_handlers::get_tour).put(tour_handlers::update_tour).delete(tour_handlers::delete_tour))
        .route("/{id}/restore", post(tour_handlers::restore_tour));

    let transporte_routes = Router::new()
        .route("/", get(transporte_handlers::list_transportes).post(transporte_handlers::create_transporte))
        .route("/with-vehicles", get(transporte_handlers::list_transportes_with_vehicles))
        .route("/{id}", get(transporte_handlers::get_transporte).put(transporte_handlers::update_transporte).delete(transporte_handlers::delete_transporte))
        .route("/{id}/restore", post(transporte_handlers::restore_transporte));

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
        .route("/{id}/restore", post(restaurante_handlers::restore_restaurante));

    let entrada_routes = Router::new()
        .route("/", get(entrada_handlers::list_entradas).post(entrada_handlers::create_entrada))
        .route("/search", get(entrada_handlers::search_entradas))
        .route("/{id}", get(entrada_handlers::get_entrada).put(entrada_handlers::update_entrada).delete(entrada_handlers::delete_entrada))
        .route("/{id}/restore", post(entrada_handlers::restore_entrada));

    let file_routes = Router::new()
        .route("/", get(file_handlers::list_files).post(file_handlers::create_file))
        .route("/upcoming", get(file_handlers::list_files_upcoming))
        .route("/pending-payment", get(file_handlers::list_files_pending_payment))
        .route("/by-date", get(file_handlers::list_files_by_date_range))
        .route("/agencia/{agencia_id}", get(file_handlers::list_files_by_agencia))
        .route("/{id}", get(file_handlers::get_file).put(file_handlers::update_file).delete(file_handlers::delete_file));

    let pago_routes = Router::new()
        .route("/", get(pago_handlers::list_pagos).post(pago_handlers::create_pago))
        .route("/file/{file_id}", get(pago_handlers::list_pagos_by_file))
        .route("/file/{file_id}/balance", get(pago_handlers::get_file_balance))
        .route("/{id}", get(pago_handlers::get_pago).put(pago_handlers::update_pago).delete(pago_handlers::delete_pago));

    let user_routes = Router::new()
        .route("/", get(user_handlers::list_users));

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
        .nest("/pagos", pago_routes)
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Router principal
    Router::new()
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
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

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
