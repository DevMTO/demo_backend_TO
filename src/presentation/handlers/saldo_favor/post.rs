//! POST handlers para Saldo a Favor

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::{CancelarFileRequest, UsarSaldoFavorRequest, CancelacionResponse, MovimientoSaldoFavorResponse};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_created;

/// Helper para verificar si el usuario tiene rol autorizado
fn has_saldo_favor_access(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador)
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

    // Validar porcentaje
    if let Some(p) = request.porcentaje_penalidad {
        if p < 0.0 || p > 100.0 {
            return Err(ApplicationError::Validation(
                "El porcentaje de penalidad debe estar entre 0 y 100".to_string(),
            ));
        }
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

    // AgenciasContador solo puede usar el saldo de su agencia
    if auth.user.role == UserRole::AgenciasContador {
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
