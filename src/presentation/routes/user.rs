//! Rutas de usuarios

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::user;

use super::state::AppState;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(user::list_users).post(user::create_user))
        .route("/{id}", get(user::get_user).put(user::update_user).delete(user::delete_user))
        .route("/{id}/hard-delete", delete(user::hard_delete_user))
        .route("/{id}/activate", patch(user::activate_user))
        .route("/{id}/deactivate", patch(user::deactivate_user))
        .route("/{id}/change-password", patch(user::admin_change_password))
}
