//! POST handlers para Contabilidad

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::{
    CreateMovimientoRequest, RegistrarPagoFileRequest, VerificarPagoFileRequest,
    CreatePagoProveedorRequest, MarcarPagoProveedorPagadoRequest,
    CreateTarifaServicioRequest,
};
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_created, json_ok};

/// Helper para verificar si el usuario tiene rol de admin
fn is_admin_or_operador(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

// ============================================================================
// MOVIMIENTOS HANDLERS
// ============================================================================

/// POST /api/contabilidad/movimientos
/// Crear movimiento manual (ajuste)
#[instrument(skip(state, auth, request))]
pub async fn create_movimiento(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateMovimientoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo el admin puede crear movimientos manuales".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .create_movimiento(request, Some(auth.user.id))
        .await?;

    Ok(json_created(response))
}

// ============================================================================
// PAGOS DE FILES HANDLERS
// ============================================================================

/// POST /api/contabilidad/pagos-files/registrar
/// Registrar pago de file (agencia sube comprobante)
#[instrument(skip(state, auth, request))]
pub async fn registrar_pago_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RegistrarPagoFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // TODO: Verificar que la agencia solo puede registrar pagos de sus propios files

    let response = state
        .container
        .contabilidad_service
        .registrar_pago_file(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(response))
}

/// POST /api/contabilidad/pagos-files/verificar
/// Verificar pago de file (admin verifica)
#[instrument(skip(state, auth, request))]
pub async fn verificar_pago_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<VerificarPagoFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin_or_operador(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para verificar pagos".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .verificar_pago_file(request, auth.user.id)
        .await?;

    Ok(json_ok(response))
}

// ============================================================================
// PAGOS A PROVEEDORES HANDLERS
// ============================================================================

/// POST /api/contabilidad/pagos-proveedores
/// Crear pago a proveedor (al asignar servicio)
#[instrument(skip(state, auth, request))]
pub async fn create_pago_proveedor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreatePagoProveedorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin_or_operador(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para crear pagos a proveedores".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .create_pago_proveedor(request, Some(auth.user.id))
        .await?;

    Ok(json_created(response))
}

/// POST /api/contabilidad/pagos-proveedores/:id/pagar
/// Marcar pago a proveedor como pagado
#[instrument(skip(state, auth, request))]
pub async fn marcar_pago_proveedor_pagado(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<MarcarPagoProveedorPagadoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo el admin puede marcar pagos como pagados".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .marcar_pago_proveedor_pagado(id, request, auth.user.id)
        .await?;

    Ok(json_ok(response))
}

// ============================================================================
// TARIFAS HANDLERS
// ============================================================================

/// POST /api/contabilidad/tarifas
/// Crear tarifa
#[instrument(skip(state, auth, request))]
pub async fn create_tarifa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTarifaServicioRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo el admin puede crear tarifas".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .create_tarifa(request, Some(auth.user.id))
        .await?;

    Ok(json_created(response))
}
