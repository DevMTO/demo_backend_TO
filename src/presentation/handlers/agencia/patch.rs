//! PATCH handlers para Agencias

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::{UpdateAgenciaInterfazRequest, AgenciaResponse};
use crate::domain::entities::{Agencia, EntityType, UserRole};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_message};

/// PATCH /api/v1/agencias/mi-agencia/interfaz - Actualizar solo interfaz de mi agencia
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_agencia_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mi_agencia = state.container.agencia_service
        .get_mi_agencia(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    let old_agencia = state.container.agencia_repository
        .find_by_id(mi_agencia.id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Agencia no encontrada".to_string()))?;
    
    let updated = request.apply_to(old_agencia.clone(), Some(auth.user.id));
    let result = state.container.agencia_repository.update(&updated).await?;
    
    let _ = state.container.logging_service.log_update::<Agencia>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::Agencia,
        mi_agencia.id,
        Some(&old_agencia),
        Some(&result),
        Some(vec!["paleta_colores".to_string(), "media".to_string()]),
        None,
    ).await;
    
    Ok(json_ok(AgenciaResponse::from(result)))
}

/// PATCH /api/v1/agencias/:id/restore - Restaurar agencia desactivada
#[instrument(skip(state, auth))]
pub async fn restore_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar agencias".to_string()));
    }
    
    let _response = state.container.agencia_service
        .restore_agencia(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Agencia restaurada correctamente"))
}
