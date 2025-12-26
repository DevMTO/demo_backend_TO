//! # Auth Middleware
//! 
//! Middleware para verificación de autenticación.


use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;

/// Middleware para verificar autenticación
pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApplicationError> {
    // Intentar obtener token del header Authorization o de la cookie
    let token = extract_token_from_request(&request);
    
    match token {
        Some(token) => {
            // Verificar el token
            let _verification = state.container.verify_session_use_case
                .execute(&token)
                .await?;
            
            // Continuar con la siguiente capa
            Ok(next.run(request).await)
        }
        None => {
            Err(ApplicationError::SessionRequired)
        }
    }
}

/// Extraer token del request (header o cookie)
fn extract_token_from_request(request: &Request) -> Option<String> {
    // Primero intentar desde el header Authorization
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    // Luego intentar desde las cookies
    if let Some(cookie_header) = request.headers().get("Cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie in cookies_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "access_token" {
                    return Some(parts[1].to_string());
                }
            }
        }
    }
    
    None
}
