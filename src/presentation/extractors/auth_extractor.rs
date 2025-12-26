//! # Auth Extractor
//! 
//! Extractor para obtener el usuario autenticado del request via cookie de sesión.
//! NO usamos JWT - solo cookies HttpOnly con tokens de sesión opacos.


use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::domain::{entities::User, errors::ApplicationError};
use crate::presentation::routes::AppState;

/// Cookie name for session token (debe coincidir con auth_handlers)
const SESSION_COOKIE_NAME: &str = "__Host-session";
const SESSION_COOKIE_NAME_DEV: &str = "session";

/// Usuario autenticado extraído del request
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub session_id: Uuid,
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
        let token = extract_session_token(parts, state.config.cookie_secure)
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

/// Extraer token de sesión de la cookie HttpOnly
fn extract_session_token(parts: &Parts, secure_cookie: bool) -> Option<String> {
    // El nombre de la cookie depende del entorno
    let cookie_name = if secure_cookie {
        SESSION_COOKIE_NAME // __Host-session en producción
    } else {
        SESSION_COOKIE_NAME_DEV // session en desarrollo
    };
    
    // Obtener el header Cookie
    let cookie_header = parts.headers.get("Cookie")?;
    let cookies_str = cookie_header.to_str().ok()?;
    
    // Parsear cookies y buscar la de sesión
    for cookie in cookies_str.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            // Verificar ambos nombres por compatibilidad
            if name == cookie_name || name == SESSION_COOKIE_NAME || name == SESSION_COOKIE_NAME_DEV {
                return Some(value.to_string());
            }
        }
    }
    
    None
}

/// Extractor opcional - no falla si no hay autenticación
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApplicationError;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(auth_user) => Ok(OptionalAuthUser(Some(auth_user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
