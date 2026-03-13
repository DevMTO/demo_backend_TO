//! Rutas de cadenas hoteleras

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::cadena_hotelera;

use super::state::AppState;

pub fn cadena_hotelera_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(cadena_hotelera::list_cadenas).post(cadena_hotelera::create_cadena))
        .route("/mi-cadena", get(cadena_hotelera::get_mi_cadena))
        .route("/mi-cadena/interfaz", patch(cadena_hotelera::patch_mi_cadena_interfaz))
        .route("/{id}", get(cadena_hotelera::get_cadena).put(cadena_hotelera::update_cadena).delete(cadena_hotelera::delete_cadena))
        .route("/{id}/restore", patch(cadena_hotelera::restore_cadena))
        .route("/{id}/hard-delete", delete(cadena_hotelera::hard_delete_cadena))
}
