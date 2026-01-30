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
        .route("/guia/{file_tour_id}/confirm", post(my_files::confirm_guia_assignment))
        .route("/conductor", get(my_files::get_my_files_as_conductor))
        .route("/conductor/{file_tour_id}/confirm", post(my_files::confirm_conductor_assignment))
        .route("/restaurante", get(my_files::get_my_files_as_restaurante))
}
