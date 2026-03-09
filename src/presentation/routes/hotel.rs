//! Rutas de hoteles

use axum::{
    Router,
    routing::{get, patch, delete},
};

use crate::presentation::handlers::hotel;

use super::state::AppState;

pub fn hotel_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(hotel::list_hoteles).post(hotel::create_hotel))
        .route("/mi-hotel", get(hotel::get_mi_hotel).put(hotel::update_mi_hotel))
        .route("/cadena/{id_cadena}", get(hotel::list_hoteles_by_cadena))
        .route("/{id}", get(hotel::get_hotel).put(hotel::update_hotel).delete(hotel::delete_hotel))
        .route("/{id}/restore", patch(hotel::restore_hotel))
        .route("/{id}/hard-delete", delete(hotel::hard_delete_hotel))
}
