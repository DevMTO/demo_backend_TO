//! Rutas de storage (Tigris)

use axum::{
    Router,
    routing::{get, post},
};

use crate::presentation::handlers::storage;

use super::state::AppState;

pub fn storage_routes() -> Router<AppState> {
    Router::new()
        .route("/agencia/{agencia_id}/logo", post(storage::upload_agencia_logo).delete(storage::delete_agencia_logo))
        .route("/agencia/{agencia_id}/banner", post(storage::upload_agencia_banner).delete(storage::delete_agencia_banner))
        .route("/cadena/{cadena_id}/logo", post(storage::upload_cadena_logo).delete(storage::delete_cadena_logo))
        .route("/transporte/{transporte_id}/logo", post(storage::upload_transporte_logo))
        .route("/tour/{tour_id}/image", post(storage::upload_tour_image))
        .route("/proxy/{*file_path}", get(storage::proxy_file))
}
