//! Rutas directas de precios de entrada (CRUD individual)

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::entrada_precio;

use super::state::AppState;

pub fn entrada_precio_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(entrada_precio::create_precio))
        .route("/{id}", get(entrada_precio::get_precio)
            .put(entrada_precio::update_precio)
            .delete(entrada_precio::delete_precio))
}
