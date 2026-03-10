use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreateTarifaRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// Crear una nueva tarifa
#[instrument(skip(state, auth, body))]
pub async fn create_tarifa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateTarifaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Solo admin puede crear tarifas
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden gestionar tarifas".to_string()
        ));
    }

    body.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let tarifa = state.container.tarifa_service
        .create_tarifa(body, Some(auth.user.id))
        .await?;

    Ok(json_created(tarifa))
}
