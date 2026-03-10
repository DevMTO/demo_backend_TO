//! Rutas de tarifas

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::tarifa;

use super::state::AppState;

pub fn tarifa_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(tarifa::create_tarifa))
        .route("/by-tour/{id_tour}", get(tarifa::list_tarifas_by_tour))
        .route("/by-tour/{id_tour}/tipo", get(tarifa::get_tarifa_by_tour_tipo))
        .route("/{id}", get(tarifa::get_tarifa).put(tarifa::update_tarifa).delete(tarifa::delete_tarifa))
}
