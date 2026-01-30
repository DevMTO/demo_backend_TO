//! Rutas de health check

use axum::{
    Router,
    routing::get,
};

use crate::presentation::handlers::health_check;

use super::state::AppState;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
}
