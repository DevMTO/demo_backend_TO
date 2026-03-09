//! GET handlers para Saldo a Favor

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

use super::query_params::SaldoFavorQueryParams;

/// Helper: ¿es admin/superadmin/contador?
fn is_admin(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador)
}

/// Helper: ¿es agencia y pertenece a esta agencia?
fn is_own_agencia(auth: &AuthUser, id_entidad: i32) -> bool {
    matches!(auth.user.role, UserRole::Agencias | UserRole::AgenciasContador | UserRole::AgenciasGerente)
        && auth.user.id_entidad == Some(id_entidad)
}

// ============================================================================
// RESUMEN Y DASHBOARD
// ============================================================================

/// GET /api/contabilidad/saldos-favor/resumen/{id_entidad}
/// Obtiene el resumen de saldo a favor de una agencia
#[instrument(skip(state, auth))]
pub async fn get_saldo_resumen(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_entidad): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin(&auth.user.role) && !is_own_agencia(&auth, id_entidad) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver este resumen".to_string(),
        ));
    }

    let resumen = state
        .container
        .saldo_favor_service
        .get_saldo_agencia(id_entidad)
        .await?;

    Ok(json_ok(resumen))
}

/// GET /api/contabilidad/saldos-favor/dashboard/{id_entidad}
/// Dashboard completo de saldo a favor
#[instrument(skip(state, auth))]
pub async fn get_saldo_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_entidad): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin(&auth.user.role) && !is_own_agencia(&auth, id_entidad) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver este dashboard".to_string(),
        ));
    }

    let dashboard = state
        .container
        .saldo_favor_service
        .get_dashboard(id_entidad)
        .await?;

    Ok(json_ok(dashboard))
}

/// GET /api/contabilidad/saldos-favor/todos
/// Lista saldos de todas las agencias (solo admin)
#[instrument(skip(state, auth))]
pub async fn list_all_saldos(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden ver todos los saldos".to_string(),
        ));
    }

    let saldos = state
        .container
        .saldo_favor_service
        .list_all_saldos()
        .await?;

    Ok(json_ok(saldos))
}

// ============================================================================
// CANCELACIONES
// ============================================================================

/// GET /api/contabilidad/saldos-favor/cancelaciones
/// Lista cancelaciones con filtro opcional por agencia
#[instrument(skip(state, auth))]
pub async fn list_cancelaciones(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<SaldoFavorQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let id_entidad = if is_admin(&auth.user.role) {
        params.id_entidad
    } else {
        auth.user.id_entidad
    };

    let offset = (params.page - 1) * params.page_size;
    let cancelaciones = state
        .container
        .saldo_favor_service
        .list_cancelaciones(id_entidad, params.page_size, offset)
        .await?;

    Ok(json_ok(cancelaciones))
}

// ============================================================================
// NO-SHOWS
// ============================================================================

/// GET /api/contabilidad/saldos-favor/no-shows
/// Lista no-shows con filtro opcional por agencia
#[instrument(skip(state, auth))]
pub async fn list_no_shows(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<SaldoFavorQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let id_entidad = if is_admin(&auth.user.role) {
        params.id_entidad
    } else {
        auth.user.id_entidad
    };

    let offset = (params.page - 1) * params.page_size;
    let no_shows = state
        .container
        .saldo_favor_service
        .list_no_shows(id_entidad, params.page_size, offset)
        .await?;

    Ok(json_ok(no_shows))
}

// ============================================================================
// MOVIMIENTOS
// ============================================================================

/// GET /api/contabilidad/saldos-favor/movimientos
/// Lista movimientos de saldo (créditos y débitos)
#[instrument(skip(state, auth))]
pub async fn list_movimientos(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<SaldoFavorQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    let id_entidad = if is_admin(&auth.user.role) {
        params.id_entidad
    } else {
        auth.user.id_entidad
    };

    let offset = (params.page - 1) * params.page_size;
    let movimientos = state
        .container
        .saldo_favor_service
        .list_movimientos(id_entidad, params.page_size, offset)
        .await?;

    Ok(json_ok(movimientos))
}
