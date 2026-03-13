//! Chat handlers - Agregar y obtener notas de files y file_tours

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::{instrument, warn};

use crate::application::dtos::chat_dto::{ChatNoteRequest, UpdateChatNoteRequest};
use crate::application::services::chat_service::ChatUserInfo;
use crate::domain::entities::{
    NotificationCategory, NotificationPriority, NotificationType, UserRole,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Helper: ¿puede acceder a chat? (admin, agencia, hotel, gerentes, contadores)
fn can_access_chat(role: &UserRole) -> bool {
    matches!(role, 
        UserRole::SuperAdmin | UserRole::Admin | 
        UserRole::Agencias | UserRole::AgenciasGerente | UserRole::AgenciasContador |
        UserRole::Hoteles | UserRole::HotelesGerente
    )
}

async fn check_file_access(state: &AppState, auth: &AuthUser, file_id: i32) -> Result<(), ApplicationError> {
    // First check if user has allowed role
    if !can_access_chat(&auth.user.role) {
        return Err(ApplicationError::Forbidden("No tienes permiso para acceder al chat".to_string()));
    }

    // Admin can access any file
    if auth.user.role.is_admin() {
        return Ok(());
    }

    let file = state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;

    let user_entidad = auth.user.id_entidad.unwrap_or(0);
    
    if file.id_entidad == user_entidad {
        return Ok(());
    }

    // HotelesGerente: verificar si el file pertenece a un hotel de su cadena
    if auth.user.role == UserRole::HotelesGerente {
        if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(file.id_entidad).await {
            if hotel.id_cadena == user_entidad {
                return Ok(());
            }
        }
    }

    Err(ApplicationError::Forbidden("No tienes acceso a este file".to_string()))
}

async fn check_file_tour_access(state: &AppState, auth: &AuthUser, file_tour_id: i32) -> Result<(), ApplicationError> {
    // First check if user has allowed role
    if !can_access_chat(&auth.user.role) {
        return Err(ApplicationError::Forbidden("No tienes permiso para acceder al chat".to_string()));
    }

    // Admin can access any file_tour
    if auth.user.role.is_admin() {
        return Ok(());
    }

    let file_tour = state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

    let file = state.container.file_repository
        .find_by_id(file_tour.id_file)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("File no encontrado".to_string()))?;

    let user_entidad = auth.user.id_entidad.unwrap_or(0);

    // Acceso directo por entidad
    if file.id_entidad == user_entidad {
        return Ok(());
    }

    // HotelesGerente: verificar si el file pertenece a un hotel de su cadena
    if auth.user.role == UserRole::HotelesGerente {
        if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(file.id_entidad).await {
            if hotel.id_cadena == user_entidad {
                return Ok(());
            }
        }
    }

    Err(ApplicationError::Forbidden("No tienes acceso a este file tour".to_string()))
}

#[instrument(skip(state, auth, request))]
pub async fn chat_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
    Json(request): Json<ChatNoteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_access(&state, &auth, file_id).await?;

    let user_info = ChatUserInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };

    let updated_notas = state.container.chat_service
        .chat_file(
            file_id,
            Some(request.nota),
            Some(user_info),
        )
        .await?;

    Ok(json_ok(serde_json::json!({ "notas": updated_notas })))
}

#[instrument(skip(state, auth))]
pub async fn get_chat_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_access(&state, &auth, file_id).await?;

    let notas = state.container.chat_service
        .get_chat_file(file_id)
        .await?;

    Ok(json_ok(serde_json::json!({ "notas": notas })))
}

#[instrument(skip(state, auth, request))]
pub async fn chat_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_tour_id): Path<i32>,
    Json(request): Json<ChatNoteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_tour_access(&state, &auth, file_tour_id).await?;

    let user_info = ChatUserInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };

    let updated_notas = state.container.chat_service
        .chat_file_tour(
            file_tour_id,
            Some(request.nota.clone()),
            Some(user_info),
        )
        .await?;

    // Notificación SSE — SOLO para notas de file_tours (mensajería)
    // Las notas de files NO generan notificaciones (es otro apartado)
    {
        // Obtener contexto del file_tour y su file para el mensaje
        let file_tour = state.container.file_tour_repository
            .find_by_id(file_tour_id)
            .await?;
        let file_code = if let Some(ref ft) = file_tour {
            state.container.file_repository
                .find_by_id(ft.id_file)
                .await?
                .and_then(|f| f.file_code.clone())
                .unwrap_or_else(|| format!("File #{}", ft.id_file))
        } else {
            format!("FileTour #{}", file_tour_id)
        };

        let nota_preview = if request.nota.len() > 80 {
            format!("{}...", &request.nota[..80])
        } else {
            request.nota.clone()
        };

        if let Err(e) = state
            .notify_roles_with_broadcast(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                &format!("Nueva nota en {}", file_code),
                &format!(
                    "{} agregó una nota al tour #{}: {}",
                    auth.user.username, file_tour_id, nota_preview
                ),
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Normal,
                Some(auth.user.id),
            )
            .await
        {
            warn!("Error al notificar nota de file_tour: {}", e);
        }
    }

    Ok(json_ok(serde_json::json!({ "notas": updated_notas })))
}

#[instrument(skip(state, auth))]
pub async fn get_chat_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_tour_access(&state, &auth, file_tour_id).await?;

    let notas = state.container.chat_service
        .get_chat_file_tour(file_tour_id)
        .await?;

    Ok(json_ok(serde_json::json!({ "notas": notas })))
}

/// Actualiza una nota específica en un file_tour por note_id
#[instrument(skip(state, auth, request))]
pub async fn update_chat_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((file_tour_id, note_id)): Path<(i32, String)>,
    Json(request): Json<UpdateChatNoteRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_tour_access(&state, &auth, file_tour_id).await?;

    let user_info = ChatUserInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };

    let updated_notas = state.container.chat_service
        .update_note_file_tour(
            file_tour_id,
            &note_id,
            &request.nota,
            Some(user_info),
        )
        .await?;

    Ok(json_ok(serde_json::json!({ "notas": updated_notas })))
}

/// Elimina una nota específica de un file_tour por note_id
#[instrument(skip(state, auth))]
pub async fn delete_chat_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((file_tour_id, note_id)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApplicationError> {
    check_file_tour_access(&state, &auth, file_tour_id).await?;

    let user_info = ChatUserInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };

    let updated_notas = state.container.chat_service
        .delete_note_file_tour(
            file_tour_id,
            &note_id,
            Some(user_info),
        )
        .await?;

    Ok(json_ok(serde_json::json!({ "notas": updated_notas })))
}
