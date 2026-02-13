//! Rutas de "Mis Files" (para guías, conductores, restaurantes)
//! Cada usuario ve solo los files donde está asignado

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::my_files;

use super::state::AppState;

pub fn my_files_routes() -> Router<AppState> {
    Router::new()
        .route("/guia", get(my_files::get_my_files_as_guia))
        .route("/conductor", get(my_files::get_my_files_as_conductor))
        .route("/restaurante", get(my_files::get_my_files_as_restaurante))
        .route("/guia/:id/confirmar", post(my_files::confirmar_asignacion_guia))
        .route("/conductor/:id/confirmar", post(my_files::confirmar_asignacion_conductor))
}
