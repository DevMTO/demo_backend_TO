//! GET handlers para Saldo a Favor

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::application::dtos::{SaldoFavorDashboard, SaldoFavorResponse, CancelacionResponse, MovimientoSaldoFavorResponse, NoShowResponse};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

use super::query_params::{CancelacionesQueryParams, MovimientosSaldoQueryParams, NoShowsQueryParams};

/// Helper para verificar si el usuario tiene rol autorizado para saldos a favor
fn has_saldo_favor_access(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador)
}

/// Helper para verificar si es admin general
fn is_admin(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

// ============================================================================
// DASHBOARD
// ============================================================================

/// GET /api/v1/saldos-favor/dashboard/{id_agencia}
/// Obtiene el dashboard de saldo a favor para una agencia específica
#[instrument(skip(state, auth))]
pub async fn get_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_agencia): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver saldos a favor".to_string(),
        ));
    }

    // Agencias_contador solo puede ver su propia agencia
    if auth.user.role == UserRole::AgenciasContador {
        if auth.user.id_entidad != Some(id_agencia) {
            return Err(ApplicationError::Forbidden(
                "Solo puedes ver el saldo de tu propia agencia".to_string(),
            ));
        }
    }

    let dashboard: SaldoFavorDashboard = state.container.saldo_favor_service.get_dashboard(id_agencia).await?;
    Ok(json_ok(dashboard))
}

// ============================================================================
// SALDOS
// ============================================================================

/// GET /api/v1/saldos-favor
/// Lista todos los saldos de todas las agencias (admin) o de la propia agencia
#[instrument(skip(state, auth))]
pub async fn list_saldos(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver saldos a favor".to_string(),
        ));
    }

    if is_admin(&auth.user.role) {
        let saldos: Vec<SaldoFavorResponse> = state.container.saldo_favor_service.list_all_saldos().await?;
        Ok(json_ok(saldos))
    } else {
        // AgenciasContador: solo sus propios saldos
        let id_agencia = auth.user.id_entidad
            .ok_or_else(|| ApplicationError::Forbidden("No tienes agencia asignada".to_string()))?;
        let saldo: SaldoFavorResponse = state.container.saldo_favor_service.get_saldo_agencia(id_agencia).await?;
        Ok(json_ok(vec![saldo]))
    }
}

/// GET /api/v1/saldos-favor/agencia/{id_agencia}
/// Obtiene el saldo de una agencia específica
#[instrument(skip(state, auth))]
pub async fn get_saldo_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_agencia): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver saldos a favor".to_string(),
        ));
    }

    if auth.user.role == UserRole::AgenciasContador && auth.user.id_entidad != Some(id_agencia) {
        return Err(ApplicationError::Forbidden(
            "Solo puedes ver el saldo de tu propia agencia".to_string(),
        ));
    }

    let saldo: SaldoFavorResponse = state.container.saldo_favor_service.get_saldo_agencia(id_agencia).await?;
    Ok(json_ok(saldo))
}

// ============================================================================
// CANCELACIONES
// ============================================================================

/// GET /api/v1/saldos-favor/cancelaciones
/// Lista cancelaciones con filtros
#[instrument(skip(state, auth))]
pub async fn list_cancelaciones(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<CancelacionesQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver cancelaciones".to_string(),
        ));
    }

    let id_agencia = if is_admin(&auth.user.role) {
        params.id_agencia
    } else {
        // AgenciasContador solo ve sus cancelaciones
        Some(auth.user.id_entidad
            .ok_or_else(|| ApplicationError::Forbidden("No tienes agencia asignada".to_string()))?)
    };

    let cancelaciones: Vec<CancelacionResponse> = state.container.saldo_favor_service
        .list_cancelaciones(id_agencia, params.page, params.page_size)
        .await?;
    Ok(json_ok(cancelaciones))
}

// ============================================================================
// MOVIMIENTOS
// ============================================================================

/// GET /api/v1/saldos-favor/movimientos
/// Lista movimientos de saldo a favor
#[instrument(skip(state, auth))]
pub async fn list_movimientos(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<MovimientosSaldoQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver movimientos de saldo".to_string(),
        ));
    }

    let id_agencia = if is_admin(&auth.user.role) {
        params.id_agencia
    } else {
        Some(auth.user.id_entidad
            .ok_or_else(|| ApplicationError::Forbidden("No tienes agencia asignada".to_string()))?)
    };

    let movimientos: Vec<MovimientoSaldoFavorResponse> = state.container.saldo_favor_service
        .list_movimientos(id_agencia, params.tipo.as_deref(), params.page, params.page_size)
        .await?;
    Ok(json_ok(movimientos))
}

// ============================================================================
// NO SHOWS
// ============================================================================

/// GET /api/v1/saldos-favor/no-shows
/// Lista no-shows con filtros
#[instrument(skip(state, auth))]
pub async fn list_no_shows(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<NoShowsQueryParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para ver no-shows".to_string(),
        ));
    }

    let id_agencia = if is_admin(&auth.user.role) {
        params.id_agencia
    } else {
        Some(auth.user.id_entidad
            .ok_or_else(|| ApplicationError::Forbidden("No tienes agencia asignada".to_string()))?)
    };

    let no_shows: Vec<NoShowResponse> = state.container.saldo_favor_service
        .list_no_shows(id_agencia, params.page, params.page_size)
        .await?;
    Ok(json_ok(no_shows))
}
