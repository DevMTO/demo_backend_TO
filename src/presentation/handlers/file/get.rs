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
    Query(params): Query<PaginationParams>,
    Query(entidad_query): Query<EntidadQuery>
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
    let mut final_agencia_id = agencia_id;
    if !auth.user.role.is_admin() {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        
        let check_cadena = if auth.user.role == crate::domain::entities::UserRole::HotelesGerenteCadena {
            if query.entidad.as_deref() == Some("cadenas_hoteleras") {
                // Si la consulta es por cadena completa, validamos la propiedad en el repositorio directo usando user_entidad
                final_agencia_id = user_entidad;
                true
            } else if query.entidad.as_deref() == Some("hoteles") {
                // Verificar en la BD si el hotel (agencia_id) pertenece a la cadena (user_entidad)
                if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(agencia_id).await {
                    hotel.id_cadena == user_entidad
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if user_entidad != final_agencia_id && !check_cadena {
            return Err(ApplicationError::Forbidden(
                "No tienes permiso para ver files de otra entidad".to_string()
            ));
        }
    }

    // Para non-admin, forzar entidad desde su rol o permitir la consulta de su jerarquía
    let entidad_filter = if auth.user.role.is_admin() {
        query.entidad.as_deref()
    } else if auth.user.role == crate::domain::entities::UserRole::HotelesGerenteCadena {
        if query.entidad.as_deref() == Some("cadenas_hoteleras") {
            Some("cadenas_hoteleras")
        } else {
            Some("hoteles")
        }
    } else {
        auth.user.role.entidad_type()
    };

    let files = state.container.file_service
        .list_files_by_agencia(final_agencia_id, entidad_filter)
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

/// Obtener file_codes de files activos (no completado/cancelado/no_show/anulado)
/// Scoped por entidad del usuario autenticado
#[instrument(skip(state, auth))]
pub async fn get_active_file_codes(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<EntidadQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let id_entidad = if auth.user.role.is_admin() {
        return Err(ApplicationError::BadRequest(
            "Admins deben especificar la entidad via /active-codes/{entidad_id}".to_string()
        ));
    } else {
        auth.user.id_entidad.ok_or_else(|| {
            ApplicationError::Forbidden("Usuario sin entidad asignada".to_string())
        })?
    };

    let entidad_filter = if auth.user.role == crate::domain::entities::UserRole::HotelesGerenteCadena {
        if query.entidad.as_deref() == Some("cadenas_hoteleras") {
            Some("cadenas_hoteleras")
        } else {
            Some("hoteles")
        }
    } else {
        auth.user.role.entidad_type()
    };
    let codes = state.container.file_service
        .get_active_file_codes(id_entidad, entidad_filter)
        .await?;

    Ok(json_ok(codes))
}

/// Obtener file_codes de files activos para una entidad específica (admin)
#[instrument(skip(state, auth))]
pub async fn get_active_file_codes_by_entity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(entidad_id): Path<i32>,
    Query(query): Query<EntidadQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mut final_entidad_id = entidad_id;
    if !auth.user.role.is_admin() {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        
        let check_cadena = if auth.user.role == crate::domain::entities::UserRole::HotelesGerenteCadena {
            if query.entidad.as_deref() == Some("cadenas_hoteleras") {
                final_entidad_id = user_entidad;
                true
            } else if query.entidad.as_deref() == Some("hoteles") {
                if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(entidad_id).await {
                    hotel.id_cadena == user_entidad
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if user_entidad != final_entidad_id && !check_cadena {
            return Err(ApplicationError::Forbidden(
                "No tienes permiso para ver file codes de otra entidad".to_string()
            ));
        }
    }

    let entidad_filter = if auth.user.role.is_admin() {
        query.entidad.as_deref()
    } else if auth.user.role == crate::domain::entities::UserRole::HotelesGerenteCadena {
        if query.entidad.as_deref() == Some("cadenas_hoteleras") {
            Some("cadenas_hoteleras")
        } else {
            Some("hoteles")
        }
    } else {
        auth.user.role.entidad_type()
    };

    let codes = state.container.file_service
        .get_active_file_codes(final_entidad_id, entidad_filter)
        .await?;

    Ok(json_ok(codes))
}
