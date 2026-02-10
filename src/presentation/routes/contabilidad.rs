//! Rutas de contabilidad

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::contabilidad;

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
        .route("/pagos-proveedores", get(contabilidad::list_pagos_proveedores).post(contabilidad::create_pago_proveedor))
        .route("/pagos-proveedores/{id}/pagar", post(contabilidad::marcar_pago_proveedor_pagado))
}