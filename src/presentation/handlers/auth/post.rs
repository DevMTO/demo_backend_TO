//! POST handlers para Auth

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_cookies::Cookies;
use tracing::{info, instrument};

use crate::application::dtos::auth_dto::{
    AuthResponse, LoginRequest, LogoutRequest,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

use super::helpers::{create_session_cookie, remove_session_cookie};

/// POST /api/v1/auth/login - Iniciar sesión
#[instrument(skip(state, cookies, payload), fields(identifier = %payload.identifier))]
pub async fn login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Intento de login para: {}", payload.identifier);
    
    let output = state.container.login_use_case
        .execute(payload, None, None)
        .await?;
    
    let cookie_max_age = state.container.cookie_max_age_hours * 3600;
    let session_cookie = create_session_cookie(&output.session_token, cookie_max_age, &state.container);
    cookies.add(session_cookie);
    
    info!("Login exitoso para: {}", output.user_info.username);
    
    let response = AuthResponse::new(
        output.user_info,
        output.session_id,
        output.expires_in_seconds,
        false,
    );
    
    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/v1/auth/logout - Cerrar sesión
#[instrument(skip(state, cookies, auth_user))]
pub async fn logout_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
    Json(payload): Json<Option<LogoutRequest>>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Logout para usuario: {}", auth_user.user.username);
    
    let logout_request = payload.unwrap_or(LogoutRequest { all_sessions: false });
    
    state.container.logout_use_case
        .execute(auth_user.user.id, auth_user.session_id, logout_request)
        .await?;
    
    let removal_cookie = remove_session_cookie(&state.container);
    cookies.add(removal_cookie);
    
    info!("Logout exitoso para: {}", auth_user.user.username);
    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Logout exitoso"}))))
}
