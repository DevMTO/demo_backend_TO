//! Rutas de entradas

use axum::{
    Router,
    routing::{get, post, patch, delete},
};

use crate::presentation::handlers::{entrada, entrada_precio};

use super::state::AppState;

pub fn entrada_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(entrada::list_entradas).post(entrada::create_entrada))
        .route("/search", get(entrada::search_entradas))
        .route("/{id}", get(entrada::get_entrada).put(entrada::update_entrada).delete(entrada::delete_entrada))
        .route("/{id}/hard-delete", delete(entrada::hard_delete_entrada))
        .route("/{id}/restore", patch(entrada::restore_entrada))
        // Precios de entrada por ID de entrada
        .route("/{id}/precios", get(entrada_precio::list_precios_by_entrada))
        .route("/{id}/precios/tipo/{tipo_precio}", get(entrada_precio::list_precios_by_tipo))
        .route("/{id}/precios/batch", post(entrada_precio::create_precios_batch))
        .route("/{id}/precios/replace", axum::routing::put(entrada_precio::replace_all_precios))
        .route("/{id}/precios/initialize", post(entrada_precio::initialize_default_precios))
        .route("/{id}/calcular-precio", get(entrada_precio::calcular_precio))
}
