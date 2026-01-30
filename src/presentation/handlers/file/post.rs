//! POST handlers para File

use axum::{extract::State, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::CreateFileRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// Crear nuevo file
/// 
/// Si el usuario tiene rol "agencias", se auto-asigna su agencia (id_entidad).
/// Si el usuario es superadmin/admin, debe proporcionar id_agencia en el request.
#[instrument(skip(state, auth, request))]
pub async fn create_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.file_service
        .create_file(
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
            auth.user.role.clone(),
            auth.user.id_entidad,
        )
        .await?;
    
    Ok(json_created(response))
}
