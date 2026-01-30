//! Rutas de guías

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::guia;

use super::state::AppState;

pub fn guia_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(guia::list_guias).post(guia::create_guia))
        .route("/search", get(guia::search_guias))
        .route("/available", get(guia::list_guias_available))
        .route("/{id}", get(guia::get_guia).put(guia::update_guia).delete(guia::delete_guia))
        .route("/{id}/restore", patch(guia::restore_guia))
        .route("/{id}/hard-delete", delete(guia::hard_delete_guia))
}
