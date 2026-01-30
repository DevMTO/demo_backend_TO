//! Rutas de contabilidad

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::contabilidad;

use super::state::AppState;

pub fn contabilidad_routes() -> Router<AppState> {
    Router::new()
        // Dashboards
        .route("/dashboard/admin", get(contabilidad::get_admin_dashboard))
        .route("/dashboard/agencia/{id_agencia}", get(contabilidad::get_agencia_dashboard))
        // Movimientos
        .route("/movimientos", get(contabilidad::list_movimientos).post(contabilidad::create_movimiento))
        // Pagos de files (agencias -> operador)
        .route("/pagos-files", get(contabilidad::list_pagos_files))
        .route("/pagos-files/registrar", post(contabilidad::registrar_pago_file))
        .route("/pagos-files/verificar", post(contabilidad::verificar_pago_file))
        // Pagos a proveedores (operador -> transportes/restaurantes/guías)
        .route("/pagos-proveedores", get(contabilidad::list_pagos_proveedores).post(contabilidad::create_pago_proveedor))
        .route("/pagos-proveedores/{id}/pagar", post(contabilidad::marcar_pago_proveedor_pagado))
        // Tarifas
        .route("/tarifas", get(contabilidad::list_tarifas).post(contabilidad::create_tarifa))
        .route("/tarifas/{id}", axum::routing::put(contabilidad::update_tarifa).delete(contabilidad::deactivate_tarifa))
}
