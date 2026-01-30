//! Rutas de autenticación

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::{
    login_handler,
    logout_handler,
    verify_session_handler,
    get_profile_handler,
    update_profile_handler,
};

use super::state::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify", get(verify_session_handler))
        .route("/me", get(verify_session_handler))
        .route("/profile", get(get_profile_handler).put(update_profile_handler))
}
