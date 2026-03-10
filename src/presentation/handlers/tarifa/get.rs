use axum::{extract::{Path, Query, State}, response::IntoResponse};
use serde::Deserialize;
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Listar tarifas de un tour
#[instrument(skip(state, _auth))]
pub async fn list_tarifas_by_tour(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_tour): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let tarifas = state.container.tarifa_service
        .get_tarifas_by_tour(id_tour)
        .await?;
    Ok(json_ok(tarifas))
}

/// Obtener tarifa por ID
#[instrument(skip(state, _auth))]
pub async fn get_tarifa(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let tarifa = state.container.tarifa_service
        .get_tarifa(id)
        .await?;
    Ok(json_ok(tarifa))
}

#[derive(Debug, Deserialize)]
pub struct TarifaTipoQuery {
    pub tipo_entidad: String,
}

/// Obtener tarifa de un tour por tipo de entidad (query param)
#[instrument(skip(state, _auth))]
pub async fn get_tarifa_by_tour_tipo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_tour): Path<i32>,
    Query(query): Query<TarifaTipoQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let tarifa = state.container.tarifa_service
        .get_tarifa_by_tour_and_tipo(id_tour, &query.tipo_entidad)
        .await?;
    Ok(json_ok(tarifa))
}
