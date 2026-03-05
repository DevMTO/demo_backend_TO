//! Rutas de contabilidad

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::contabilidad;
use crate::presentation::handlers::saldo_favor;

use super::state::AppState;

pub fn contabilidad_routes() -> Router<AppState> {
    Router::new()
        // Dashboard agencia
        .route("/dashboard/agencia/{id_agencia}", get(contabilidad::get_agencia_dashboard))
        // Pagos de files (agencias -> operador)
        .route("/pagos-files", get(contabilidad::list_pagos_files))
        .route("/pagos-files/registrar", post(contabilidad::registrar_pago_file))
        .route("/pagos-files/verificar", post(contabilidad::verificar_pago_file))
        // Pagos a proveedores (operador -> transportes/restaurantes/guias)
        .route("/pagos-proveedores/{id}/pagar", post(contabilidad::marcar_pago_proveedor_pagado))
        // Saldo a Favor - Consultas
        .route("/saldos-favor/resumen/{id_agencia}", get(saldo_favor::get_saldo_resumen))
        .route("/saldos-favor/dashboard/{id_agencia}", get(saldo_favor::get_saldo_dashboard))
        .route("/saldos-favor/todos", get(saldo_favor::list_all_saldos))
        .route("/saldos-favor/cancelaciones", get(saldo_favor::list_cancelaciones))
        .route("/saldos-favor/no-shows", get(saldo_favor::list_no_shows))
        .route("/saldos-favor/movimientos", get(saldo_favor::list_movimientos))
        // Saldo a Favor - Acciones
        .route("/saldos-favor/cancelar-file", post(saldo_favor::cancelar_file))
        .route("/saldos-favor/cancelar-tour", post(saldo_favor::cancelar_file_tour))
        .route("/saldos-favor/registrar-no-show", post(saldo_favor::registrar_no_show))
        .route("/saldos-favor/registrar-no-show-tour", post(saldo_favor::registrar_no_show_file_tour))
        .route("/saldos-favor/autorizar-saldo", post(saldo_favor::autorizar_no_show_saldo))
        .route("/saldos-favor/usar-saldo", post(saldo_favor::usar_saldo))
}
