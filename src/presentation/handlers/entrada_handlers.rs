use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateEntradaRequest, UpdateEntradaRequest, EntradaResponse};
use crate::domain::errors::ApplicationError;

use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_message};

#[derive(Debug, serde::Deserialize, Default)]
pub struct EntradaListParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(default)]
    pub include_inactive: bool,
}

impl EntradaListParams {
    pub fn to_options(&self) -> crate::application::ports::PaginationOptions {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(50).clamp(1, 500);
        let offset = (page - 1) * page_size;
        crate::application::ports::PaginationOptions { limit: Some(page_size), offset: Some(offset) }
    }
}

#[instrument(skip(state, _auth))]
pub async fn list_entradas(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<EntradaListParams>) -> Result<impl IntoResponse, ApplicationError> {
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
pub async fn get_entrada(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let e = state.container.entrada_service.get_entrada(id).await?;
    Ok(json_ok(EntradaResponse::from(e)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_entrada(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateEntradaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let entity = request.into_entity(Some(auth.user.id));
    let created = state.container.entrada_service.create_entrada(&entity, auth.user.id, &auth.user.username).await?;
    
    // Inicializar precios por defecto para la nueva entrada
    let _ = state.container.entrada_precio_service
        .initialize_default_precios(created.id, Some(auth.user.id))
        .await;
    
    Ok(json_created(EntradaResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateEntradaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_e = state.container.entrada_service.get_entrada(id).await?;
    let updated = request.apply_to(old_e, Some(auth.user.id));
    let result = state.container.entrada_service.update_entrada(id, &updated, auth.user.id, &auth.user.username).await?;
    Ok(json_ok(EntradaResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_service.deactivate_entrada(id, auth.user.id, &auth.user.username).await?;
    Ok(json_message("Entrada desactivada"))
}

#[instrument(skip(state, auth))]
pub async fn restore_entrada(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_service.restore_entrada(id, auth.user.id, &auth.user.username).await?;
    Ok(json_message("Entrada restaurada"))
}

#[derive(Debug, serde::Deserialize)]
pub struct EntradaSearchQuery { pub ruta: Option<String> }

#[instrument(skip(state, _auth))]
pub async fn search_entradas(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<EntradaSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let entradas = if let Some(ruta) = query.ruta {
        state.container.entrada_service.search_by_ruta(&ruta).await?
    } else {
        state.container.entrada_service.list_entradas(Default::default()).await?.data
    };
    Ok(json_ok(entradas.into_iter().map(EntradaResponse::from).collect::<Vec<_>>()))
}

