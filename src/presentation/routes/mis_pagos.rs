//! Rutas de Mis Pagos
//!
//! Endpoints para que los proveedores (guías, conductores, restaurantes)
//! consulten sus pagos pendientes y pagados.

use axum::{
    Router,
    routing::get,
};

use crate::presentation::handlers::mis_pagos;

use super::state::AppState;

pub fn mis_pagos_routes() -> Router<AppState> {
    Router::new()
        // Mis pagos para guías
        .route("/guia", get(mis_pagos::get_mis_pagos_guia))
        // Mis pagos para conductores
        .route("/conductor", get(mis_pagos::get_mis_pagos_conductor))
        // Mis pagos para restaurantes
        .route("/restaurante", get(mis_pagos::get_mis_pagos_restaurante))
}
