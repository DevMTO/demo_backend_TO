//! PUT handlers para Agencias

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::UpdateAgenciaRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/v1/agencias/:id - Actualizar agencia completa
#[instrument(skip(state, auth, request))]
pub async fn update_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para actualizar agencias".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.agencia_service
        .update_agencia(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(response))
}

/// PUT /api/v1/agencias/mi-agencia - Actualizar mi propia agencia (campos limitados)
#[instrument(skip(state, auth, request))]
pub async fn update_mi_agencia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateAgenciaRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mi_agencia = state.container.agencia_service
        .get_mi_agencia(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    // Solo permitir ciertos campos
    let limited_request = UpdateAgenciaRequest {
        nombre: None,
        ruc: None,
        direccion: request.direccion,
        telefono: request.telefono,
        correo: request.correo,
        encargado: None,
        paleta_colores: request.paleta_colores,
        media: request.media,
        is_active: None,
        pago_anticipado: None,
        tipo_vencimiento: None,
    };
    
    let response = state.container.agencia_service
        .update_agencia(
            mi_agencia.id,
            limited_request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}
