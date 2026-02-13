//! Handlers POST para My Files
//! Confirmación de lectura de asignaciones por guías y conductores

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{instrument, warn};

use crate::application::dtos::ConfirmAssignmentResponse;
use crate::domain::entities::{
    NotificationCategory, NotificationPriority, NotificationType, UserRole,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::routes::AppState;

#[instrument(skip(state, auth))]
pub async fn confirmar_asignacion_guia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::Guias {
        return Err(ApplicationError::Forbidden(
            "Solo los guías pueden confirmar esta asignación".to_string(),
        ));
    }

    let _id_persona = auth.user.id_persona.ok_or_else(|| {
        ApplicationError::Validation("Usuario no tiene persona asociada".to_string())
    })?;

    let _file_guia = state
        .container
        .file_guia_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| {
            ApplicationError::NotFound(format!("Asignación de guía {} no encontrada", id))
        })?;

    if let Err(e) = state
        .container
        .notification_service
        .notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Asignación confirmada",
            &format!(
                "{} (guía) ha confirmado la lectura de su asignación #{}",
                auth.user.username, id
            ),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(auth.user.id),
        )
        .await
    {
        warn!("Error al notificar confirmación de guía: {}", e);
    }

    Ok(json_ok(ConfirmAssignmentResponse {
        success: true,
        mensaje: "Asignación confirmada correctamente".to_string(),
        estado_confirmacion: "leido".to_string(),
        confirmado_at: Some(Utc::now()),
    }))
}

#[instrument(skip(state, auth))]
pub async fn confirmar_asignacion_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if auth.user.role != UserRole::Conductores {
        return Err(ApplicationError::Forbidden(
            "Solo los conductores pueden confirmar esta asignación".to_string(),
        ));
    }

    let _id_persona = auth.user.id_persona.ok_or_else(|| {
        ApplicationError::Validation("Usuario no tiene persona asociada".to_string())
    })?;

    let _file_vehiculo = state
        .container
        .file_vehiculo_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| {
            ApplicationError::NotFound(format!("Asignación de conductor {} no encontrada", id))
        })?;

    if let Err(e) = state
        .container
        .notification_service
        .notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Asignación confirmada",
            &format!(
                "{} (conductor) ha confirmado la lectura de su asignación #{}",
                auth.user.username, id
            ),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(auth.user.id),
        )
        .await
    {
        warn!("Error al notificar confirmación de conductor: {}", e);
    }

    Ok(json_ok(ConfirmAssignmentResponse {
        success: true,
        mensaje: "Asignación confirmada correctamente".to_string(),
        estado_confirmacion: "leido".to_string(),
        confirmado_at: Some(Utc::now()),
    }))
}
