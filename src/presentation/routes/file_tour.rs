//! Rutas de file-tours (relaciones entre files y tours)

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::presentation::handlers::{chat, file_relations, relation_status};

use super::state::AppState;

pub fn file_tour_routes() -> Router<AppState> {
    Router::new()
        // Entradas
        .route(
            "/{file_tour_id}/entradas",
            get(file_relations::list_file_tour_entradas),
        )
        .route(
            "/entradas",
            post(file_relations::assign_entrada_to_file_tour),
        )
        .route(
            "/entradas/{entrada_asig_id}",
            axum::routing::delete(file_relations::remove_file_entrada),
        )
        .route(
            "/entradas/{id}/status",
            patch(relation_status::update_file_entrada_status),
        )
        // Restaurantes
        .route(
            "/{file_tour_id}/restaurantes",
            get(file_relations::list_file_tour_restaurantes),
        )
        .route(
            "/restaurantes",
            post(file_relations::assign_restaurante_to_file_tour),
        )
        .route(
            "/restaurantes/{restaurante_asig_id}",
            axum::routing::delete(file_relations::remove_file_restaurante),
        )
        .route(
            "/restaurantes/{id}/status",
            patch(relation_status::update_file_restaurante_status),
        )
        // Guías
        .route(
            "/{file_tour_id}/guias",
            get(file_relations::list_file_tour_guias),
        )
        .route("/guias", post(file_relations::assign_guia_to_file_tour))
        .route(
            "/guias/{guia_asig_id}",
            axum::routing::delete(file_relations::remove_file_guia)
                .put(file_relations::update_file_guia),
        )
        .route(
            "/guias/{id}/status",
            patch(relation_status::update_file_guia_status),
        )
        // Vehículos
        .route(
            "/{file_tour_id}/vehiculos",
            get(file_relations::list_file_tour_vehiculos),
        )
        .route(
            "/vehiculos",
            post(file_relations::assign_vehiculo_to_file_tour),
        )
        .route(
            "/vehiculos/{vehiculo_asig_id}",
            axum::routing::delete(file_relations::remove_file_vehiculo),
        )
        .route(
            "/vehiculos/{id}/status",
            patch(relation_status::update_file_vehiculo_status_relation),
        )
        .route(
            "/{file_tour_id}/vehiculos/{vehiculo_id}/status",
            patch(file_relations::update_vehiculo_status),
        )
        // File Tour status y recojo
        .route(
            "/{id}/status",
            patch(relation_status::update_file_tour_status),
        )
        .route(
            "/{id}/hora-recojo",
            patch(relation_status::update_file_tour_hora_recojo),
        )
        .route(
            "/{id}/recojo",
            patch(relation_status::update_file_tour_recojo),
        )
        // Chat/Notas
        .route(
            "/{id}/notas",
            get(chat::get_chat_file_tour).post(chat::chat_file_tour),
        )
        .route(
            "/{id}/notas/{note_id}",
            put(chat::update_chat_file_tour).delete(chat::delete_chat_file_tour),
        )
}
