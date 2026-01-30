//! GET handlers para Entrada

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use tracing::instrument;

use crate::application::dtos::EntradaResponse;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginatedResponse, PaginationInfo, json_ok};

use super::query_params::EntradaListParams;

#[instrument(skip(state, _auth))]
pub async fn list_entradas(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Query(params): Query<EntradaListParams>
) -> Result<impl IntoResponse, ApplicationError> {
    let result = if params.include_inactive {
        state.container.entrada_service.list_all_entradas(params.to_options()).await?
    } else {
        state.container.entrada_service.list_entradas(params.to_options()).await?
    };
    let page = result.current_page();
    let page_size = result.limit;
    let total_pages = result.pages();
    Ok(json_ok(PaginatedResponse {
        items: result.data.into_iter().map(EntradaResponse::from).collect(),
        pagination: PaginationInfo { page, page_size, total: result.total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_entrada(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let e = state.container.entrada_service.get_entrada(id).await?;
    Ok(json_ok(EntradaResponse::from(e)))
}

/// Search/list entradas (simplified - no longer searches by ruta)
#[instrument(skip(state, _auth))]
pub async fn search_entradas(
    State(state): State<AppState>, 
    _auth: AuthUser
) -> Result<impl IntoResponse, ApplicationError> {
    let entradas = state.container.entrada_service.list_entradas(Default::default()).await?.data;
    Ok(json_ok(entradas.into_iter().map(EntradaResponse::from).collect::<Vec<_>>()))
}
