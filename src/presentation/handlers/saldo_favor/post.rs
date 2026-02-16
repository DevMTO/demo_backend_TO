//! POST handlers para Saldo a Favor

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::{CancelarFileRequest, UsarSaldoFavorRequest, RegistrarNoShowRequest, CancelacionResponse, MovimientoSaldoFavorResponse, NoShowResponse, CancelarFileTourRequest, NoShowFileTourRequest};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_created;

/// Helper para verificar si el usuario tiene rol autorizado
fn has_saldo_favor_access(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador | UserRole::Agencias | UserRole::AgenciasGerente)
}

// ============================================================================
// CANCELAR FILE
// ============================================================================

/// POST /api/v1/saldos-favor/cancelar
/// Cancela un file y genera saldo a favor
#[instrument(skip(state, auth))]
pub async fn cancelar_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CancelarFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar files".to_string(),
        ));
    }

    let cancelacion: CancelacionResponse = state.container.saldo_favor_service
        .cancelar_file(request, auth.user.id)
        .await?;
    Ok(json_created(cancelacion))
}

// ============================================================================
// USAR SALDO
// ============================================================================

/// POST /api/v1/saldos-favor/usar
/// Usa saldo a favor para pagar un file
#[instrument(skip(state, auth))]
pub async fn usar_saldo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UsarSaldoFavorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para usar saldo a favor".to_string(),
        ));
    }

    // Agencias solo pueden usar el saldo de su propia agencia
    if matches!(auth.user.role, UserRole::AgenciasContador | UserRole::Agencias | UserRole::AgenciasGerente) {
        if auth.user.id_entidad != Some(request.id_agencia) {
            return Err(ApplicationError::Forbidden(
                "Solo puedes usar el saldo de tu propia agencia".to_string(),
            ));
        }
    }

    if request.monto <= 0.0 {
        return Err(ApplicationError::Validation(
            "El monto debe ser mayor a 0".to_string(),
        ));
    }

    let movimiento: MovimientoSaldoFavorResponse = state.container.saldo_favor_service
        .usar_saldo(request, auth.user.id)
        .await?;
    Ok(json_created(movimiento))
}

// ============================================================================
// REGISTRAR NO-SHOW (Solo admin)
// ============================================================================

/// POST /api/v1/saldos-favor/no-show
/// Registra un no-show para un file (solo admin, después de 8PM)
#[instrument(skip(state, auth))]
pub async fn registrar_no_show(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RegistrarNoShowRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden registrar no-shows
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo los administradores pueden registrar no-shows".to_string(),
        ));
    }

    let no_show: NoShowResponse = state.container.saldo_favor_service
        .registrar_no_show(request, auth.user.id)
        .await?;
    Ok(json_created(no_show))
}

// ============================================================================
// CANCELAR FILE TOUR (individual tour dentro de un file)
// ============================================================================

/// POST /api/v1/saldos-favor/cancelar-tour
/// Cancela un file_tour individual y genera saldo a favor por las entradas/restaurantes del tour
#[instrument(skip(state, auth))]
pub async fn cancelar_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CancelarFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !has_saldo_favor_access(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar tours".to_string(),
        ));
    }

    let cancelacion: CancelacionResponse = state.container.saldo_favor_service
        .cancelar_file_tour(request, auth.user.id)
        .await?;
    Ok(json_created(cancelacion))
}

// ============================================================================
// REGISTRAR NO-SHOW FILE TOUR (Solo admin, tour individual)
// ============================================================================

/// POST /api/v1/saldos-favor/no-show-tour
/// Registra un no-show para un file_tour individual (solo admin)
#[instrument(skip(state, auth))]
pub async fn registrar_no_show_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<NoShowFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo SuperAdmin y Admin pueden registrar no-shows
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo los administradores pueden registrar no-shows de tours".to_string(),
        ));
    }

    let no_show: NoShowResponse = state.container.saldo_favor_service
        .registrar_no_show_file_tour(request, auth.user.id)
        .await?;
    Ok(json_created(no_show))
}
