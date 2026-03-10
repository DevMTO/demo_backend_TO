use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::UpdateTarifaRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Actualizar una tarifa existente
#[instrument(skip(state, auth, body))]
pub async fn update_tarifa(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    auth: AuthUser,
    Json(body): Json<UpdateTarifaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden gestionar tarifas".to_string()
        ));
    }

    body.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let tarifa = state.container.tarifa_service
        .update_tarifa(id, body, Some(auth.user.id))
        .await?;

    Ok(json_ok(tarifa))
}
