//! GET handlers para EntradaPrecio

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use tracing::instrument;

use crate::application::dtos::EntradaPrecioResponse;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;
use super::query_params::CalcularPrecioQuery;

/// GET /api/entradas/:id_entrada/precios
/// Obtener todos los precios de una entrada
#[instrument(skip(state, _auth))]
pub async fn list_precios_by_entrada(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_entrada): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precios = state.container.entrada_precio_service
        .get_precios_by_entrada(id_entrada)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = precios
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_ok(response))
}

/// GET /api/entradas/:id_entrada/precios/tipo/:tipo_precio
/// Obtener precios de una entrada por tipo (general, nacional, extranjero)
#[instrument(skip(state, _auth))]
pub async fn list_precios_by_tipo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((id_entrada, tipo_precio)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precios = state.container.entrada_precio_service
        .get_precios_by_tipo(id_entrada, &tipo_precio)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = precios
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_ok(response))
}

/// GET /api/entrada-precios/:id
/// Obtener un precio específico por ID
#[instrument(skip(state, _auth))]
pub async fn get_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precio = state.container.entrada_precio_service.get_precio(id).await?;
    Ok(json_ok(EntradaPrecioResponse::from(precio)))
}

/// GET /api/entradas/:id_entrada/calcular-precio?edad=25&tipo_turista=nacional
/// Calcular el precio aplicable para una entrada según edad y tipo de turista
#[instrument(skip(state, _auth))]
pub async fn calcular_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_entrada): Path<i32>,
    Query(query): Query<CalcularPrecioQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precio = state.container.entrada_precio_service
        .calcular_precio(id_entrada, query.edad, &query.tipo_turista)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "id_entrada": id_entrada,
        "edad": query.edad,
        "tipo_turista": query.tipo_turista,
        "precio": precio.to_string()
    })))
}
