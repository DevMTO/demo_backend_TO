//! Rutas de restaurantes

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::restaurante;

use super::state::AppState;

pub fn restaurante_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(restaurante::list_restaurantes).post(restaurante::create_restaurante))
        .route("/search", get(restaurante::search_restaurantes))
        .route("/{id}", get(restaurante::get_restaurante).put(restaurante::update_restaurante).delete(restaurante::delete_restaurante))
        .route("/{id}/hard-delete", delete(restaurante::hard_delete_restaurante))
        .route("/{id}/restore", patch(restaurante::restore_restaurante))
}
