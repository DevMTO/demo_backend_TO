//! Rutas de conductores

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::conductor;

use super::state::AppState;

pub fn conductor_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(conductor::list_conductores).post(conductor::create_conductor))
        .route("/available", get(conductor::list_conductores_available))
        .route("/transporte/{transporte_id}", get(conductor::list_conductores_by_transporte))
        .route("/{id}", get(conductor::get_conductor).put(conductor::update_conductor).delete(conductor::delete_conductor))
        .route("/{id}/restore", patch(conductor::restore_conductor))
        .route("/{id}/hard-delete", delete(conductor::hard_delete_conductor))
}
