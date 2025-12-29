use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::domain::{entities::User, errors::ApplicationError};
use crate::presentation::routes::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub session_id: i32,
    /// Token rotado si la sesión fue renovada automáticamente
    pub rotated_token: Option<String>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApplicationError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extraer token de la cookie de sesión
        let token = extract_session_token(parts, &state.container.cookie_name)
            .ok_or(ApplicationError::SessionRequired)?;
        
        // Verificar sesión (incluye rotación automática si es necesario)
        let verification = state.container.verify_session_use_case
            .execute(&token)
            .await?;
        
        Ok(AuthUser {
            user: verification.user,
            session_id: verification.session_id,
            rotated_token: verification.new_token,
        })
    }
}

fn extract_session_token(parts: &Parts, cookie_name: &str) -> Option<String> {
    // Obtener el header Cookie
    let cookie_header = parts.headers.get("Cookie")?;
    let cookies_str = cookie_header.to_str().ok()?;
    
    // Parsear cookies y buscar la de sesión
    for cookie in cookies_str.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name == cookie_name {
                return Some(value.to_string());
            }
        }
    }
    
    None
}
