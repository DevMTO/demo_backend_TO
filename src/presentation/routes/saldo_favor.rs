//! Rutas para Saldo a Favor

use axum::{Router, routing::{get, post}};
use crate::presentation::handlers::saldo_favor;
use super::state::AppState;

pub fn saldo_favor_routes() -> Router<AppState> {
    Router::new()
        // Saldos
        .route("/", get(saldo_favor::list_saldos))
        .route("/agencia/{id_agencia}", get(saldo_favor::get_saldo_agencia))
        .route("/dashboard/{id_agencia}", get(saldo_favor::get_dashboard))
        // Cancelaciones
        .route("/cancelar", post(saldo_favor::cancelar_file))
        .route("/cancelar-tour", post(saldo_favor::cancelar_file_tour))
        .route("/cancelaciones", get(saldo_favor::list_cancelaciones))
        // No-shows
        .route("/no-show", post(saldo_favor::registrar_no_show))
        .route("/no-show-tour", post(saldo_favor::registrar_no_show_file_tour))
        .route("/no-shows", get(saldo_favor::list_no_shows))
        // Movimientos
        .route("/usar", post(saldo_favor::usar_saldo))
        .route("/movimientos", get(saldo_favor::list_movimientos))
}
