//! PUT handlers para File

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::UpdateFileRequest;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Actualizar file existente
#[instrument(skip(state, auth, request))]
pub async fn update_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(mut request): Json<UpdateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    if let Some(new_notas) = &request.notas {
        if !new_notas.is_empty() {
            let existing_file = state.container.file_service
                .get_file(id)
                .await?;

            let existing_notas: JsonValue = existing_file.notas
                .as_ref()
                .and_then(|n| serde_json::from_str(n).ok())
                .unwrap_or(json!({}));

            let timestamp = Utc::now().to_rfc3339();
            let mut merged_notas = existing_notas;
            merged_notas[timestamp.clone()] = json!(new_notas);

            request.notas = serde_json::to_string(&merged_notas).ok();
        }
    }
    
    // Validar permisos
    let file = state.container.file_service.get_file(id).await?;
    if !auth.user.role.is_admin() {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        let check_cadena = if auth.user.role == UserRole::HotelesGerente {
            if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(file.id_entidad).await {
                hotel.id_cadena == user_entidad
            } else {
                false
            }
        } else {
            false
        };

        if file.id_entidad != user_entidad && !check_cadena {
            return Err(ApplicationError::Forbidden("No tienes permiso para actualizar este file".to_string()));
        }
    }
    
    let response = state.container.file_service
        .update_file(
            id, 
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}
