//! # Auth Handlers
//! 
//! Handlers para endpoints de autenticación con cookies de sesión.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use tower_cookies::Cookies;
use tracing::{info, warn, debug, instrument};

use crate::application::dtos::auth_dto::{
    LoginRequest, LogoutRequest, AuthResponse,
    SuccessResponse, AuthUserInfo,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

/// Login handler - crea sesión y establece cookie segura
#[instrument(skip(state, cookies, request), fields(identifier = %request.identifier))]
pub async fn login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🔐 Intento de login para: {}", request.identifier);
    
    // Extraer IP y User-Agent del request
    let ip_address = None;
    let user_agent = None;
    
    // Ejecutar caso de uso
    debug!("Ejecutando LoginUseCase...");
    let output = match state.container.login_use_case
        .execute(request.clone(), ip_address, user_agent)
        .await {
            Ok(output) => {
                info!("✅ Login exitoso para usuario: {} (id: {})", output.user_info.username, output.user_info.id);
                output
            },
            Err(e) => {
                warn!("❌ Login fallido para {}: {:?}", request.identifier, e);
                return Err(e);
            }
        };
    
    // Configurar cookie de sesión
    debug!("Configurando cookie de sesión...");
    debug!("Cookie config - name: {}, path: {}, http_only: {}, secure: {}, same_site: {}", 
        state.container.cookie_name,
        state.container.cookie_path,
        state.container.cookie_http_only,
        state.container.cookie_secure,
        state.container.cookie_same_site
    );
    
    let session_cookie = create_session_cookie(
        &output.session_token,
        output.expires_in_seconds,
        &state.container,
    );
    
    info!("🍪 Cookie creada: name={}, max_age={}s", 
        state.container.cookie_name, 
        output.expires_in_seconds
    );
    
    cookies.add(session_cookie);
    
    // Construir respuesta
    let auth_response = AuthResponse::new(
        output.user_info,
        output.session_id,
        output.expires_in_seconds,
        request.remember_me,
    );
    
    info!("🎉 Login completo, sesión creada: {}", output.session_id);
    
    Ok((StatusCode::OK, Json(auth_response)))
}

/// Logout handler - invalida sesión y limpia cookies
#[instrument(skip(state, cookies, auth_user, request))]
pub async fn logout_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
    Json(request): Json<LogoutRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚪 Logout para usuario: {} (sesión: {})", auth_user.user.username, auth_user.session_id);
    
    // Ejecutar caso de uso
    let count = state.container.logout_use_case
        .execute(auth_user.user.id, auth_user.session_id, request)
        .await?;
    
    // Limpiar cookie de sesión
    remove_session_cookie(&cookies, &state.container);
    
    info!("✅ Logout completado: {} sesión(es) cerrada(s)", count);
    
    Ok((
        StatusCode::OK,
        Json(SuccessResponse::new(format!("{} sesión(es) cerrada(s)", count))),
    ))
}

/// Verify session handler - verifica que la sesión sea válida
#[instrument(skip(state, cookies, auth_user))]
pub async fn verify_session_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🔍 Verificando sesión para usuario: {}", auth_user.user.username);
    
    // Si la sesión fue rotada, actualizar la cookie
    if let Some(new_token) = auth_user.rotated_token.as_ref() {
        debug!("Token rotado, actualizando cookie...");
        let session_cookie = create_session_cookie(
            new_token,
            state.container.cookie_max_age_hours * 3600,
            &state.container,
        );
        cookies.add(session_cookie);
    }
    
    let user_info = AuthUserInfo {
        id: auth_user.user.id,
        id_persona: auth_user.user.id_persona,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        role: auth_user.user.role.to_string(),
        id_entidad: auth_user.user.id_entidad,
        nombre_entidad: auth_user.user.nombre_entidad.clone(),
        status: auth_user.user.status.to_string(),
    };
    
    info!("✅ Sesión válida para: {}", auth_user.user.username);
    
    Ok((StatusCode::OK, Json(user_info)))
}

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

/// Helper para crear cookies de sesión
fn create_session_cookie(
    token: &str,
    max_age_secs: i64,
    container: &crate::infrastructure::container::DependencyContainer,
) -> Cookie<'static> {
    let same_site = match container.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };
    
    Cookie::build((container.cookie_name.clone(), token.to_string()))
        .path(container.cookie_path.clone())
        .http_only(container.cookie_http_only)
        .secure(container.cookie_secure)
        .same_site(same_site)
        .max_age(time::Duration::seconds(max_age_secs))
        .build()
}

/// Helper para eliminar cookie de sesión
fn remove_session_cookie(
    cookies: &Cookies,
    container: &crate::infrastructure::container::DependencyContainer,
) {
    let same_site = match container.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };
    
    let removal_cookie = Cookie::build((container.cookie_name.clone(), "".to_string()))
        .path(container.cookie_path.clone())
        .http_only(container.cookie_http_only)
        .secure(container.cookie_secure)
        .same_site(same_site)
        .max_age(time::Duration::ZERO)
        .build();
    
    cookies.add(removal_cookie);
}
