//! GET handlers para Contabilidad

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use chrono::NaiveDate;
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, PaginatedResponse, PaginationInfo};
use crate::presentation::handlers::file::query_params::EntidadQuery;

use super::query_params::{PagosFilesQueryParams, PagosProveedoresQueryParams};

/// Helper para verificar si el usuario tiene rol de admin
fn is_admin_or_operador(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

/// Helper para verificar si el usuario es agencia/hotel/contador y pertenece a esa entidad
async fn is_own_agencia(state: &AppState, auth: &AuthUser, id_entidad: i32) -> bool {
    let role = &auth.user.role;
    let user_entidad = auth.user.id_entidad;

    // Si es la misma entidad
    if matches!(role, 
        UserRole::Agencias | UserRole::AgenciasContador | UserRole::AgenciasGerente |
        UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena
    ) && user_entidad == Some(id_entidad) {
        return true;
    }

    // Si es Gerente de Cadena y busca un hotel de su cadena
    if *role == UserRole::HotelesGerenteCadena {
        if let Some(id_cadena) = user_entidad {
            if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(id_entidad).await {
                return hotel.id_cadena == id_cadena;
            }
        }
    }

    false
}

// ============================================================================
// DASHBOARD HANDLERS
// ============================================================================

/// GET /api/contabilidad/dashboard/agencia/:id_entidad
/// Obtiene el dashboard de contabilidad para una agencia especifica
#[instrument(skip(state, auth))]
pub async fn get_agencia_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_entidad): Path<i32>,
    Query(query): Query<EntidadQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let is_admin = is_admin_or_operador(&auth.user.role);
    let is_own = is_own_agencia(&state, &auth, id_entidad).await;

    if !is_admin && !is_own {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver este dashboard".to_string(),
        ));
    }

    // Para roles no-admin, filtrar por tipo de entidad
    let entidad_filter = if is_admin { 
        None 
    } else if auth.user.role == UserRole::HotelesGerenteCadena {
        if query.entidad.as_deref() == Some("cadenas_hoteleras") {
            Some("cadenas_hoteleras")
        } else {
            Some("hoteles")
        }
    } else { 
        auth.user.role.entidad_type() 
    };

    let dashboard = state
        .container
        .contabilidad_service
        .get_agencia_dashboard(id_entidad, entidad_filter)
        .await?;


    Ok(json_ok(dashboard))
}

// ============================================================================
// PAGOS DE FILES HANDLERS
// ============================================================================

/// GET /api/contabilidad/pagos-files
/// Lista pagos de files con filtros
#[instrument(skip(state, auth))]
pub async fn list_pagos_files(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PagosFilesQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mut id_entidad_filter = if is_admin_or_operador(&auth.user.role) {
        params.id_entidad
    } else if matches!(auth.user.role, 
        UserRole::Agencias | UserRole::AgenciasContador | UserRole::AgenciasGerente | 
        UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena
    ) {
        auth.user.id_entidad
    } else {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver los pagos de files".to_string(),
        ));
    };

    // Si es Gerente de Cadena y se solicita un hotel específico de su cadena
    if auth.user.role == UserRole::HotelesGerenteCadena {
        if let Some(requested_id) = params.id_entidad {
            if let Some(id_cadena) = auth.user.id_entidad {
                if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(requested_id).await {
                    if hotel.id_cadena == id_cadena {
                        id_entidad_filter = Some(requested_id);
                    }
                }
            }
        }
    }

    // Para roles no-admin, filtrar por tipo de entidad del usuario
    let entidad_filter = if is_admin_or_operador(&auth.user.role) {
        None
    } else if auth.user.role == UserRole::HotelesGerenteCadena {
        if params.entidad.as_deref() == Some("cadenas_hoteleras") {
            Some("cadenas_hoteleras")
        } else {
            // En reportes de contabilidad, la tabla pagos_files guarda 'entidad' (agencias/hoteles)
            Some("hoteles")
        }
    } else {
        auth.user.role.entidad_type()
    };

    let fecha_desde = params
        .fecha_desde
        .as_ref()
        .and_then(|f| NaiveDate::parse_from_str(f, "%Y-%m-%d").ok());
    let fecha_hasta = params
        .fecha_hasta
        .as_ref()
        .and_then(|f| NaiveDate::parse_from_str(f, "%Y-%m-%d").ok());

    let offset = (params.page - 1) * params.page_size;

    let (items, total) = state
        .container
        .contabilidad_service
        .list_pagos_files(
            id_entidad_filter,
            entidad_filter,
            params.estado.as_deref(),
            fecha_desde,
            fecha_hasta,
            params.page_size,
            offset,
        )
        .await?;

    let total_pages = (total as f64 / params.page_size as f64).ceil() as i64;

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

// ============================================================================
// PAGOS A PROVEEDORES HANDLERS
// ============================================================================

/// GET /api/contabilidad/pagos-proveedores
/// Lista pagos a proveedores con filtros
#[instrument(skip(state, auth))]
pub async fn list_pagos_proveedores(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PagosProveedoresQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin_or_operador(&auth.user.role) && auth.user.role != UserRole::AgenciasContador {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver los pagos a proveedores".to_string(),
        ));
    }

    let fecha_desde = params.fecha_desde.as_ref().and_then(|f| {
        NaiveDate::parse_from_str(f, "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    });
    let fecha_hasta = params.fecha_hasta.as_ref().and_then(|f| {
        NaiveDate::parse_from_str(f, "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc())
    });

    let offset = (params.page - 1) * params.page_size;

    let (items, total) = state
        .container
        .contabilidad_service
        .list_pagos_proveedores(
            params.tipo_proveedor.as_deref(),
            params.estado.as_deref(),
            fecha_desde,
            fecha_hasta,
            params.page_size,
            offset,
        )
        .await?;

    let total_pages = (total as f64 / params.page_size as f64).ceil() as i64;

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