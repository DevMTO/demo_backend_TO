//! Rutas de agencias

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::agencia;

use super::state::AppState;

pub fn agencia_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(agencia::list_agencias).post(agencia::create_agencia))
        .route("/mi-agencia", get(agencia::get_mi_agencia).put(agencia::update_mi_agencia))
        .route("/mi-agencia/interfaz", patch(agencia::patch_mi_agencia_interfaz))
        .route("/ruc/{ruc}", get(agencia::get_agencia_by_ruc))
        .route("/{id}", get(agencia::get_agencia).put(agencia::update_agencia).delete(agencia::delete_agencia))
        .route("/{id}/restore", patch(agencia::restore_agencia))
        .route("/{id}/hard-delete", delete(agencia::hard_delete_agencia))
}
