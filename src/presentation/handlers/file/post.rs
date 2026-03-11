//! POST handlers para File

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::application::dtos::{CreateFileRequest, ConfirmReservaRequest};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::UserRole;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_created, json_ok};

/// Crear nuevo file
/// 
/// Si el usuario tiene rol "agencias", se auto-asigna su agencia (id_entidad).
/// Si el usuario es superadmin/admin, debe proporcionar id_entidad en el request.
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

/// Confirmar una reserva (file)
/// 
/// Este endpoint:
/// 1. Cambia el status del file de "reservado" a "confirmado"
/// 2. Crea un registro de pago pendiente (pagos_files)
/// 3. Notifica a los admins
/// 4. Notifica al contador de la agencia (si existe)
/// 5. Registra en el log de actividad
/// 
/// Solo puede ser usado por usuarios con rol de agencia o admin.
#[instrument(skip(state, auth, request))]
pub async fn confirmar_reserva(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<ConfirmReservaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let file_id = request.file_id;
    
    // Validar permisos
    let file = state.container.file_service.get_file(file_id).await?;
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
            return Err(ApplicationError::Forbidden("No tienes permiso para confirmar este file".to_string()));
        }
    }

    info!("📋 Confirmando reserva - File ID: {} por usuario: {}", file_id, auth.user.username);

    let response = state.container.file_service
        .confirmar_reserva(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;

    Ok(json_ok(response))
}
