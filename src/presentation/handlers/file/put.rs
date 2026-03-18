//! PUT handlers para File

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{AuditInfo, UpdateFileRequest, UpdateFileWithServicesRequest};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created};

/// Actualizar file existente
#[instrument(skip(state, auth, request))]
pub async fn update_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Path(id): Path<i32>, 
    Json(mut request): Json<UpdateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let nota = request.notas.clone();
    
    request.notas = None;
    
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

    if let Some(nota_value) = nota {
        let user_info = AuditInfo {
            user_id: auth.user.id,
            username: auth.user.username.clone(),
            is_admin: auth.user.role.is_admin(),
        };
        let _ = state.container.chat_service
            .chat_file(
                id,
                Some(nota_value),
                Some(user_info),
            )
            .await;
    }
    
    Ok(json_ok(response))
}

#[instrument(skip(state, auth, request))]
pub async fn update_file_with_services(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateFileWithServicesRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let response = state.container.file_service
        .update_file_with_services(id, request, auth.user.id)
        .await?;

    Ok(json_created(response))
}
