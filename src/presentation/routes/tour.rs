//! Rutas de tours

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::tour;

use super::state::AppState;

pub fn tour_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(tour::list_tours).post(tour::create_tour))
        .route("/search", get(tour::search_tours))
        .route("/{id}", get(tour::get_tour).put(tour::update_tour).delete(tour::delete_tour))
        .route("/{id}/hard-delete", delete(tour::hard_delete_tour))
        .route("/{id}/restore", patch(tour::restore_tour))
}
