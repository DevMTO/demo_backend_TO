//! # Auth Handlers
//! 
//! Handlers para endpoints de autenticación con cookies de sesión ultra-seguras.
//! NO usamos JWT - solo tokens de sesión opacos con HMAC y HttpOnly cookies.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use tower_cookies::Cookies;

use crate::application::dtos::{
    LoginRequest, RegisterRequest, LogoutRequest,
    AuthResponse, SuccessResponse, UserInfo,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

/// Cookie name for session token (ultra-seguro, HttpOnly, SameSite=Strict)
const SESSION_COOKIE_NAME: &str = "__Host-session";

/// Login handler - crea sesión y establece cookie segura
pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Extraer IP y User-Agent del request (simplificado)
    let ip_address = None; // TODO: extraer de headers X-Forwarded-For / X-Real-IP
    let user_agent = None; // TODO: extraer de header User-Agent
    
    // Ejecutar caso de uso
    let output = state.container.login_use_case
        .execute(request.clone(), ip_address, user_agent)
        .await?;
    
    // Configurar cookie de sesión ultra-segura
    let session_cookie = create_session_cookie(
        &output.session_token,
        output.expires_in_seconds,
        &state.config,
    );
    cookies.add(session_cookie);
    
    // Construir respuesta (el token NO se envía en el body, solo en cookie)
    let auth_response = AuthResponse::new(
        output.user_info,
        output.session_id,
        output.expires_in_seconds,
        request.remember_me,
    );
    
    Ok((StatusCode::OK, Json(auth_response)))
}

/// Register handler - crea usuario sin iniciar sesión
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let output = state.container.register_use_case
        .execute(request)
        .await?;
    
    Ok((StatusCode::CREATED, Json(output.user_info)))
}

/// Logout handler - invalida sesión y limpia cookies
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
    Json(request): Json<LogoutRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Ejecutar caso de uso
    let count = state.container.logout_use_case
        .execute(&auth_user.user.id, &auth_user.session_id, request)
        .await?;
    
    // Limpiar cookie de sesión
    remove_session_cookie(&cookies, &state.config);
    
    Ok((
        StatusCode::OK,
        Json(SuccessResponse::new(format!("{} sesión(es) cerrada(s)", count))),
    ))
}

/// Get current user handler - devuelve información del usuario autenticado
pub async fn get_current_user(
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    let user_info = UserInfo {
        id: auth_user.user.id,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        display_name: auth_user.user.display_name.clone(),
        role: auth_user.user.role.to_string(),
        email_verified: auth_user.user.email_verified,
        mfa_enabled: auth_user.user.mfa_enabled,
    };
    
    Ok((StatusCode::OK, Json(user_info)))
}

/// Verify session handler - verifica que la sesión sea válida
pub async fn verify_session(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Si la sesión fue rotada, actualizar la cookie
    if let Some(new_token) = auth_user.rotated_token.as_ref() {
        let session_cookie = create_session_cookie(
            new_token,
            state.config.session_expiration_hours * 3600,
            &state.config,
        );
        cookies.add(session_cookie);
    }
    
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "valid": true,
            "user_id": auth_user.user.id,
            "session_id": auth_user.session_id,
            "token_rotated": auth_user.rotated_token.is_some(),
        })),
    ))
}

/// Touch session handler - actualiza actividad sin verificar completamente
pub async fn touch_session(
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "session_id": auth_user.session_id,
        })),
    ))
}

/// Helper para crear cookies de sesión ultra-seguras
fn create_session_cookie<'a>(
    token: &str,
    max_age_secs: i64,
    config: &crate::config::AppConfig,
) -> Cookie<'static> {
    // SameSite siempre Strict para máxima seguridad contra CSRF
    let same_site = match config.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax, // Solo para desarrollo
        "none" => SameSite::None, // PELIGROSO - solo para casos específicos con HTTPS
        _ => SameSite::Strict, // DEFAULT: máxima seguridad
    };
    
    // Usamos __Host- prefix para máxima seguridad:
    // - Requiere HTTPS (Secure)
    // - No puede tener Domain attribute
    // - Path debe ser "/"
    let cookie_name = if config.cookie_secure {
        SESSION_COOKIE_NAME // __Host-session
    } else {
        "session" // Para desarrollo sin HTTPS
    };
    
    Cookie::build((cookie_name.to_string(), token.to_string()))
        .path("/")
        .http_only(true) // NO accesible desde JavaScript - previene XSS
        .secure(config.cookie_secure) // HTTPS only en producción
        .same_site(same_site) // Previene CSRF
        .max_age(time::Duration::seconds(max_age_secs))
        .build()
}

/// Helper para eliminar cookie de sesión
fn remove_session_cookie(cookies: &Cookies, config: &crate::config::AppConfig) {
    let same_site = match config.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };
    
    let cookie_name = if config.cookie_secure {
        SESSION_COOKIE_NAME
    } else {
        "session"
    };
    
    let removal_cookie = Cookie::build((cookie_name.to_string(), "".to_string()))
        .path("/")
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(same_site)
        .max_age(time::Duration::ZERO)
        .build();
    
    cookies.add(removal_cookie);
}
