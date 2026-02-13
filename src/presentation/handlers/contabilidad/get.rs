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

use super::query_params::{PagosFilesQueryParams, PagosProveedoresQueryParams};

/// Helper para verificar si el usuario tiene rol de admin
fn is_admin_or_operador(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador)
}

/// Helper para verificar si el usuario es agencia/contador de agencia y pertenece a esa agencia
fn is_own_agencia(auth: &AuthUser, id_agencia: i32) -> bool {
    matches!(auth.user.role, UserRole::Agencias | UserRole::AgenciasContador | UserRole::AgenciasGerente)
        && auth.user.id_entidad == Some(id_agencia)
}

// ============================================================================
// DASHBOARD HANDLERS
// ============================================================================

/// GET /api/contabilidad/dashboard/agencia/:id_agencia
/// Obtiene el dashboard de contabilidad para una agencia especifica
#[instrument(skip(state, auth))]
pub async fn get_agencia_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_agencia): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let is_admin = is_admin_or_operador(&auth.user.role);
    let is_own = is_own_agencia(&auth, id_agencia);

    if !is_admin && !is_own {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver este dashboard".to_string(),
        ));
    }

    let dashboard = state
        .container
        .contabilidad_service
        .get_agencia_dashboard(id_agencia)
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
    let id_agencia_filter = if is_admin_or_operador(&auth.user.role) {
        params.id_agencia
    } else if auth.user.role == UserRole::Agencias || auth.user.role == UserRole::AgenciasGerente {
        auth.user.id_entidad
    } else {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver los pagos de files".to_string(),
        ));
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
            id_agencia_filter,
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
    if !is_admin_or_operador(&auth.user.role) {
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