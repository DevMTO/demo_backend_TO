use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Eliminar una tarifa por ID
#[instrument(skip(state, auth))]
pub async fn delete_tarifa(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden gestionar tarifas".to_string()
        ));
    }

    state.container.tarifa_service
        .delete_tarifa(id)
        .await?;

    Ok(json_ok(serde_json::json!({ "message": "Tarifa eliminada correctamente" })))
}
