//! Rutas de activity logs

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::activity_log;

use super::state::AppState;

pub fn activity_log_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(activity_log::list_logs))
        .route("/summary", get(activity_log::get_logs_summary))
        .route("/errors", get(activity_log::get_recent_errors))
        .route("/cleanup", post(activity_log::cleanup_old_logs))
}
