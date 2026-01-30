//! DELETE handlers para Contabilidad

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_deleted;

/// DELETE /api/contabilidad/tarifas/:id
/// Desactivar tarifa
#[instrument(skip(state, auth))]
pub async fn deactivate_tarifa(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo el admin puede desactivar tarifas".to_string(),
        ));
    }

    state
        .container
        .contabilidad_service
        .deactivate_tarifa(id)
        .await?;

    Ok(json_deleted())
}
