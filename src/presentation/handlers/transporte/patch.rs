//! PATCH handlers para Transporte

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::{info, instrument};

use crate::application::dtos::UpdateTransporteInterfazRequest;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_message};

use super::helpers::find_user_transporte_id;

/// PATCH /api/v1/transportes/me/interfaz - Actualizar solo la interfaz de mi transporte
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_transporte_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateTransporteInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🎨 Usuario '{}' actualiza interfaz de su transporte", auth.user.username);
    
    let transporte_id = find_user_transporte_id(&state, &auth).await?;
    
    let result = state.container.transporte_service
        .update_interface(transporte_id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("Interfaz de transporte '{}' actualizada", result.nombre);
    Ok(json_ok(result))
}

/// POST /api/v1/transportes/:id/restore - Restaurar un transporte desactivado
#[instrument(skip(state, auth))]
pub async fn restore_transporte(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.transporte_service
        .restore_transporte(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    info!("♻️ Handler: Transporte restaurado (ID: {})", id);
    Ok(json_message("Transporte restaurado"))
}
