//! Rutas de file-vehiculos (listar todas las asignaciones vehículo-file)

use axum::{
    Router,
    routing::get,
};

use crate::presentation::handlers::file_relations;

use super::state::AppState;

pub fn file_vehiculos_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(file_relations::list_all_file_vehiculos))
}
