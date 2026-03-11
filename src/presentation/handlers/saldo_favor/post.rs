//! POST handlers para Saldo a Favor

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::{self, json, Value as JsonValue};
use tracing::instrument;

use crate::application::dtos::contabilidad_dto::{
    CancelarFileRequest, CancelarFileTourRequest,
    RegistrarNoShowRequest, NoShowFileTourRequest,
    AutorizarNoShowSaldoRequest, UsarSaldoFavorRequest,
};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

/// Helper: ¿es admin/superadmin?
fn is_admin(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

/// Helper: ¿puede gestionar contabilidad?
fn can_manage_contabilidad(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador | UserRole::Agencias | UserRole::Hoteles | UserRole::HotelesGerente)
}

// ============================================================================
// CANCELACIONES
// ============================================================================

/// POST /api/contabilidad/saldos-favor/cancelar-file
/// Cancela un file completo - todo el monto pagado se convierte en saldo a favor
#[instrument(skip(state, auth))]
pub async fn cancelar_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut request): Json<CancelarFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar files".to_string(),
        ));
    }

    let existing_file = state.container.file_service
        .get_file(request.id_file)
        .await?;

    let existing_notas: JsonValue = existing_file.notas
        .as_ref()
        .and_then(|n| serde_json::from_str(n).ok())
        .unwrap_or(json!({}));

    let new_notas = request.notas.clone().unwrap_or_else(|| "{}".to_string());
    let mut new_notas_json: JsonValue = serde_json::from_str(&new_notas).unwrap_or(json!({}));
    new_notas_json["canceled_by"] = json!(auth.user.id);
    new_notas_json["canceled_by_username"] = json!(auth.user.username.clone());

    let timestamp = Utc::now().to_rfc3339();
    let mut merged_notas = existing_notas;
    merged_notas[timestamp.clone()] = json!(new_notas_json);

    request.notas = serde_json::to_string(&merged_notas).ok();

    let result = state
        .container
        .saldo_favor_service
        .cancelar_file(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/cancelar-tour
/// Cancela un file_tour específico - parte proporcional a saldo a favor
#[instrument(skip(state, auth))]
pub async fn cancelar_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut request): Json<CancelarFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar tours".to_string(),
        ));
    }

    let notas = request.notas.clone().unwrap_or_else(|| "{}".to_string());

    let mut notas_json: JsonValue = serde_json::from_str(&notas).unwrap_or(json!({}));
    notas_json["canceled_by"] = json!(auth.user.id);
    notas_json["canceled_by_username"] = json!(auth.user.username.clone());

    let original_notas = serde_json::to_string(&notas_json).unwrap_or_default();
    let timestamp = Utc::now().to_rfc3339();
    let notas_with_timestamp = json!({ timestamp: original_notas });

    request.notas = serde_json::to_string(&notas_with_timestamp).ok();

    let result = state
        .container
        .saldo_favor_service
        .cancelar_file_tour(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}

// ============================================================================
// NO-SHOWS
// ============================================================================

/// POST /api/contabilidad/saldos-favor/registrar-no-show
/// Registra no-show de un file completo
#[instrument(skip(state, auth))]
pub async fn registrar_no_show(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RegistrarNoShowRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para registrar no-shows".to_string(),
        ));
    }

    let result = state
        .container
        .saldo_favor_service
        .registrar_no_show(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/registrar-no-show-tour
/// Registra no-show de un file_tour específico
#[instrument(skip(state, auth))]
pub async fn registrar_no_show_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<NoShowFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para registrar no-shows".to_string(),
        ));
    }

    let result = state
        .container
        .saldo_favor_service
        .registrar_no_show_file_tour(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/autorizar-saldo
/// Autoriza saldo a favor de un no-show (solo admin/superadmin)
#[instrument(skip(state, auth))]
pub async fn autorizar_no_show_saldo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AutorizarNoShowSaldoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden autorizar saldos de no-show".to_string(),
        ));
    }

    let result = state
        .container
        .saldo_favor_service
        .autorizar_no_show_saldo(request, auth.user.id)
        .await?;

    Ok(json_ok(result))
}

// ============================================================================
// USO DE SALDO
// ============================================================================

/// POST /api/contabilidad/saldos-favor/usar-saldo
/// Aplica saldo a favor al pago de un file
#[instrument(skip(state, auth))]
pub async fn usar_saldo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UsarSaldoFavorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para aplicar saldos".to_string(),
        ));
    }

    let result = state
        .container
        .saldo_favor_service
        .usar_saldo(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}
