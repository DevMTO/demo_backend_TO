//! Rutas de personas

use axum::{
    Router,
    routing::{get, delete},
};

use crate::presentation::handlers::persona;

use super::state::AppState;

pub fn persona_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(persona::list_personas).post(persona::create_persona))
        .route("/search", get(persona::search_personas))
        .route("/{id}", get(persona::get_persona).put(persona::update_persona).delete(persona::delete_persona))
        .route("/{id}/hard-delete", delete(persona::hard_delete_persona))
}
