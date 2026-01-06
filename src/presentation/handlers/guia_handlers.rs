use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateGuiaRequest, UpdateGuiaRequest, GuiaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok, json_created, json_deleted};

#[instrument(skip(state, _auth))]
pub async fn list_guias(State(state): State<AppState>, _auth: AuthUser, Query(params): Query<PaginationParams>) -> Result<impl IntoResponse, ApplicationError> {
    let page = params.page;
    let page_size = params.page_size;
    let offset = (page - 1) * page_size;
    
    let (items, total) = state.container.guia_service.list_guias_with_persona(page_size, offset).await?;
    let total_pages = (total + page_size - 1) / page_size;
    
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { page, page_size, total, total_pages },
    }))
}

#[instrument(skip(state, _auth))]
pub async fn get_guia(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    let g = state.container.guia_service.get_guia(id).await?;
    Ok(json_ok(GuiaResponse::from(g)))
}

#[instrument(skip(state, auth, request))]
pub async fn create_guia(State(state): State<AppState>, auth: AuthUser, Json(request): Json<CreateGuiaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let entity = request.into_entity(Some(auth.user.id));
    let created = state.container.guia_service.create_guia(&entity, auth.user.id, &auth.user.username).await?;
    Ok(json_created(GuiaResponse::from(created)))
}

#[instrument(skip(state, auth, request))]
pub async fn update_guia(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>, Json(request): Json<UpdateGuiaRequest>) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    let old_g = state.container.guia_service.get_guia(id).await?;
    let updated = request.apply_to(old_g, Some(auth.user.id));
    let result = state.container.guia_service.update_guia(id, &updated, auth.user.id, &auth.user.username).await?;
    Ok(json_ok(GuiaResponse::from(result)))
}

#[instrument(skip(state, auth))]
pub async fn delete_guia(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i32>) -> Result<impl IntoResponse, ApplicationError> {
    state.container.guia_service.delete_guia(id, auth.user.id, &auth.user.username).await?;
    Ok(json_deleted())
}

#[derive(Debug, serde::Deserialize)]
pub struct GuiaSearchQuery { pub idioma: Option<String>, pub especialidad: Option<String> }

#[instrument(skip(state, _auth))]
pub async fn search_guias(State(state): State<AppState>, _auth: AuthUser, Query(query): Query<GuiaSearchQuery>) -> Result<impl IntoResponse, ApplicationError> {
    let guias = if let Some(idioma) = query.idioma {
        state.container.guia_service.search_by_idioma(&idioma).await?
    } else if let Some(especialidad) = query.especialidad {
        state.container.guia_service.search_by_especialidad(&especialidad).await?
    } else {
        state.container.guia_service.list_available().await?
    };
    Ok(json_ok(guias.into_iter().map(GuiaResponse::from).collect::<Vec<_>>()))
}

#[instrument(skip(state, _auth))]
pub async fn list_guias_available(State(state): State<AppState>, _auth: AuthUser) -> Result<impl IntoResponse, ApplicationError> {
    let guias = state.container.guia_service.list_available().await?;
    Ok(json_ok(guias.into_iter().map(GuiaResponse::from).collect::<Vec<_>>()))
}
