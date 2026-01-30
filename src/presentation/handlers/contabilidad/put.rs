//! PUT handlers para Contabilidad

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::UpdateTarifaServicioRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/contabilidad/tarifas/:id
/// Actualizar tarifa
#[instrument(skip(state, auth, request))]
pub async fn update_tarifa(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTarifaServicioRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo el admin puede actualizar tarifas".to_string(),
        ));
    }

    let response = state
        .container
        .contabilidad_service
        .update_tarifa(id, request)
        .await?;

    Ok(json_ok(response))
}
