//! Rutas de files

use axum::{
    Router,
    routing::{get, post, patch, delete},
};

use crate::presentation::handlers::{file, file_relations, relation_status};

use super::state::AppState;

pub fn file_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(file::list_files).post(file::create_file))
        .route("/upcoming", get(file::list_files_upcoming))
        .route("/pending-payment", get(file::list_files_pending_payment))
        .route("/by-date", get(file::list_files_by_date_range))
        .route("/agencia/{agencia_id}", get(file::list_files_by_agencia))
        .route("/{id}", get(file::get_file).put(file::update_file).delete(file::delete_file))
        .route("/{id}/restore", patch(file::restore_file))
        .route("/{id}/hard-delete", delete(file::hard_delete_file))
        // File Relations - Pasajeros (estos siguen vinculados al file, no al file_tour)
        .route("/{id}/pasajeros", get(file_relations::list_file_pasajeros).post(file_relations::add_pasajero_to_file))
        .route("/{id}/pasajeros/bulk", post(file_relations::bulk_add_pasajeros_to_file))
        .route("/{id}/pasajeros/with-persona", post(file_relations::create_pasajero_with_persona))
        .route("/{id}/pasajeros/{pasajero_id}", axum::routing::delete(file_relations::remove_file_pasajero).put(file_relations::update_file_pasajero))
        .route("/pasajeros/{id}/status", patch(relation_status::update_file_pasajero_status))
        // File Relations - Tours (lista los tours asignados al file)
        .route("/{id}/tours", get(file_relations::list_file_tours))
}
