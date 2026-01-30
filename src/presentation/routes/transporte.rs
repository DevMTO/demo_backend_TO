//! Rutas de transportes

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::transporte;

use super::state::AppState;

pub fn transporte_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(transporte::list_transportes).post(transporte::create_transporte))
        .route("/mi-transporte", get(transporte::get_mi_transporte).put(transporte::update_mi_transporte))
        .route("/mi-transporte/interfaz", patch(transporte::patch_mi_transporte_interfaz))
        .route("/with-vehicles", get(transporte::list_transportes_with_vehicles))
        .route("/{id}", get(transporte::get_transporte).put(transporte::update_transporte).delete(transporte::delete_transporte))
        .route("/{id}/restore", patch(transporte::restore_transporte))
        .route("/{id}/hard-delete", delete(transporte::hard_delete_transporte))
}
