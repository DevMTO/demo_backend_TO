//! Módulo de seguridad HTTP
//! Provee middlewares y capas de seguridad para la API

use std::time::Duration;
use axum::{
    Router,
    body::Body,
    http::{header, Method, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::config::AppConfig;

/// Métodos HTTP permitidos en la API
const ALLOWED_METHODS: [Method; 5] = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
];

/// Crea el CORS layer con la configuración especificada
pub fn create_cors_layer(config: &AppConfig) -> CorsLayer {
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
            Method::OPTIONS, // Necesario para preflight CORS
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
            header::COOKIE,
        ])
        .expose_headers([
            header::SET_COOKIE,
            header::CONTENT_TYPE,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(config.cors_max_age_secs))
}

/// Middleware para validar métodos HTTP permitidos
/// Rechaza requests con métodos no soportados (CONNECT, TRACE, etc.)
pub async fn method_filter(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    
    // Permitir OPTIONS para CORS preflight
    if method == Method::OPTIONS {
        return Ok(next.run(request).await);
    }
    
    // Verificar si el método está en la lista de permitidos
    if ALLOWED_METHODS.contains(&method) {
        Ok(next.run(request).await)
    } else {
        tracing::warn!(
            "🚫 Método HTTP no permitido: {} - Request rechazado",
            method
        );
        Err(StatusCode::METHOD_NOT_ALLOWED)
    }
}

/// Aplica las capas de seguridad HTTP a un Router
pub fn apply_security_layers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        // Security Headers
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            header::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            header::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-xss-protection"),
            header::HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            header::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            header::HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("cache-control"),
            header::HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        ))
        // Request ID for traceability
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

/// Headers de seguridad para respuestas (constantes de referencia)
/// Se pueden usar para configuración dinámica de headers si es necesario
#[allow(dead_code)]
pub struct SecurityHeaders;

#[allow(dead_code)]
impl SecurityHeaders {
    /// X-Content-Type-Options: nosniff
    /// Previene MIME type sniffing
    pub const X_CONTENT_TYPE_OPTIONS: (&'static str, &'static str) = ("x-content-type-options", "nosniff");
    
    /// X-Frame-Options: DENY
    /// Previene clickjacking
    pub const X_FRAME_OPTIONS: (&'static str, &'static str) = ("x-frame-options", "DENY");
    
    /// X-XSS-Protection: 1; mode=block
    /// Habilita protección XSS del navegador
    pub const X_XSS_PROTECTION: (&'static str, &'static str) = ("x-xss-protection", "1; mode=block");
    
    /// Referrer-Policy: strict-origin-when-cross-origin
    /// Controla qué información de referrer se envía
    pub const REFERRER_POLICY: (&'static str, &'static str) = ("referrer-policy", "strict-origin-when-cross-origin");
    
    /// Permissions-Policy
    /// Deshabilita features del navegador innecesarias
    pub const PERMISSIONS_POLICY: (&'static str, &'static str) = ("permissions-policy", "geolocation=(), microphone=(), camera=()");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_methods_contains_standard_rest_methods() {
        assert!(ALLOWED_METHODS.contains(&Method::GET));
        assert!(ALLOWED_METHODS.contains(&Method::POST));
        assert!(ALLOWED_METHODS.contains(&Method::PUT));
        assert!(ALLOWED_METHODS.contains(&Method::PATCH));
        assert!(ALLOWED_METHODS.contains(&Method::DELETE));
    }

    #[test]
    fn test_dangerous_methods_not_allowed() {
        assert!(!ALLOWED_METHODS.contains(&Method::CONNECT));
        assert!(!ALLOWED_METHODS.contains(&Method::TRACE));
    }
}
