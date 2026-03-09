use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use tower_cookies::Cookies;
use tracing::info;

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
        
        // Si el token fue rotado, actualizar la cookie automáticamente
        if let Some(ref new_token) = verification.new_token {
            // Intentar obtener el Cookies manager de las extensiones
            if let Some(cookies) = parts.extensions.get::<Cookies>() {
                info!("Token rotado automáticamente para sesión {}, actualizando cookie...", verification.session_id);
                let session_cookie = create_session_cookie_for_rotation(
                    new_token,
                    state.container.cookie_max_age_hours,
                    &state.container.cookie_name,
                    &state.container.cookie_domain,
                    &state.container.cookie_path,
                    state.container.cookie_secure,
                    state.container.cookie_http_only,
                    &state.container.cookie_same_site,
                );
                cookies.add(session_cookie);
            }
        }
        
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

/// Crea una cookie de sesión con el token rotado
#[allow(clippy::too_many_arguments)]
fn create_session_cookie_for_rotation(
    token: &str,
    max_age_hours: i64,
    cookie_name: &str,
    cookie_domain: &str,
    cookie_path: &str,
    cookie_secure: bool,
    cookie_http_only: bool,
    cookie_same_site: &str,
) -> Cookie<'static> {
    let same_site = match cookie_same_site.to_lowercase().as_str() {
        "strict" => SameSite::Strict,
        "none" => SameSite::None,
        _ => SameSite::Lax,
    };
    
    let mut cookie = Cookie::build((cookie_name.to_string(), token.to_string()))
        .path(cookie_path.to_string())
        .max_age(time::Duration::hours(max_age_hours))
        .same_site(same_site)
        .http_only(cookie_http_only)
        .secure(cookie_secure);
    
    if !cookie_domain.is_empty() {
        cookie = cookie.domain(cookie_domain.to_string());
    }
    
    cookie.build()
}
