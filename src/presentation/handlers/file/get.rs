//! GET handlers para File

use axum::{extract::{Path, Query, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{PaginationParams, PaginatedResponse, PaginationInfo, json_ok};

use super::query_params::{DateRangeQuery, EntidadQuery};

/// Listar files con paginación
/// - Admin/SuperAdmin: ven todos los files
/// - Roles con id_entidad (agencias, hoteles, etc.): solo ven files de su entidad
#[instrument(skip(state, auth))]
pub async fn list_files(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Query(params): Query<PaginationParams>
) -> Result<impl IntoResponse, ApplicationError> {
    // Si el usuario tiene id_entidad y NO es admin, filtrar por su entidad
    if !auth.user.role.is_admin() {
        if let Some(id_entidad) = auth.user.id_entidad {
            let entidad_filter = auth.user.role.entidad_type();
            let files = state.container.file_service
                .list_files_by_agencia(id_entidad, entidad_filter)
                .await?;
            // Paginar manualmente sobre la lista filtrada
            let total = files.len() as i64;
            let page_size = params.page_size.max(1) as usize;
            let page = params.page.max(1) as usize;
            let total_pages = ((total as usize + page_size - 1) / page_size).max(1) as i64;
            let start = (page - 1) * page_size;
            let items: Vec<_> = files.into_iter().skip(start).take(page_size).collect();
            return Ok(json_ok(PaginatedResponse {
                items,
                pagination: PaginationInfo {
                    page: params.page,
                    page_size: params.page_size,
                    total,
                    total_pages,
                },
            }));
        }
    }

    let (items, total, total_pages) = state.container.file_service
        .list_files(params.to_options())
        .await?;
    
    Ok(json_ok(PaginatedResponse {
        items,
        pagination: PaginationInfo { 
            page: params.page, 
            page_size: params.page_size, 
            total, 
            total_pages,
        },
    }))
}

/// Obtener file por ID
#[instrument(skip(state, _auth))]
pub async fn get_file(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Path(id): Path<i32>
) -> Result<impl IntoResponse, ApplicationError> {
    let response = state.container.file_service
        .get_file(id)
        .await?;
    
    Ok(json_ok(response))
}

/// Listar files por agencia
/// - Admin/SuperAdmin: pueden consultar cualquier entidad
/// - Otros roles: solo pueden consultar su propia entidad
#[instrument(skip(state, auth))]
pub async fn list_files_by_agencia(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(agencia_id): Path<i32>,
    Query(query): Query<EntidadQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es admin o pertenece a la entidad solicitada
    if !auth.user.role.is_admin() {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        if user_entidad != agencia_id {
            return Err(ApplicationError::Forbidden(
                "No tienes permiso para ver files de otra entidad".to_string()
            ));
        }
    }

    // Para non-admin, forzar entidad desde su rol (evitar spoofing del query param)
    let entidad_filter = if auth.user.role.is_admin() {
        query.entidad.as_deref()
    } else {
        auth.user.role.entidad_type()
    };

    let files = state.container.file_service
        .list_files_by_agencia(agencia_id, entidad_filter)
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files por rango de fechas
#[instrument(skip(state, _auth))]
pub async fn list_files_by_date_range(
    State(state): State<AppState>, 
    _auth: AuthUser, 
    Query(query): Query<DateRangeQuery>
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .search_files_by_date_range(query.from, query.to)
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files próximos (en los próximos 7 días)
#[instrument(skip(state, _auth))]
pub async fn list_files_upcoming(
    State(state): State<AppState>, 
    _auth: AuthUser
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_upcoming()
        .await?;
    
    Ok(json_ok(files))
}

/// Listar files con pago pendiente
#[instrument(skip(state, _auth))]
pub async fn list_files_pending_payment(
    State(state): State<AppState>, 
    _auth: AuthUser
) -> Result<impl IntoResponse, ApplicationError> {
    let files = state.container.file_service
        .list_files_pending_payment()
        .await?;
    
    Ok(json_ok(files))
}
