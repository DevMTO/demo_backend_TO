//! Rutas de pagos

use axum::{
    Router,
    routing::get,
};

use crate::presentation::handlers::pago;

use super::state::AppState;

pub fn pago_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(pago::list_pagos).post(pago::create_pago))
        .route("/file/{file_id}", get(pago::list_pagos_by_file))
        .route("/file/{file_id}/balance", get(pago::get_file_balance))
        .route("/{id}", get(pago::get_pago).put(pago::update_pago).delete(pago::delete_pago))
}
