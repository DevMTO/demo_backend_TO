//! POST handlers para Pago

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreatePagoRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// Registrar nuevo pago
#[instrument(skip(state, auth, request))]
pub async fn create_pago(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreatePagoRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.pago_service
        .register_pago(
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_created(response))
}
