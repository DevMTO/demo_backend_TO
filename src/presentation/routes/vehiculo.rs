//! Rutas de vehículos

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::vehiculo;

use super::state::AppState;

pub fn vehiculo_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(vehiculo::list_vehiculos).post(vehiculo::create_vehiculo))
        .route("/available", get(vehiculo::list_vehiculos_available))
        .route("/transporte/{transporte_id}", get(vehiculo::list_vehiculos_by_transporte))
        .route("/{id}", get(vehiculo::get_vehiculo).put(vehiculo::update_vehiculo).delete(vehiculo::delete_vehiculo))
        .route("/{id}/restore", patch(vehiculo::restore_vehiculo))
        .route("/{id}/hard-delete", delete(vehiculo::hard_delete_vehiculo))
}
