//! PATCH handlers para Cadenas Hoteleras

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::{UpdateCadenaInterfazRequest, CadenaHoteleraResponse};
use crate::domain::entities::{CadenaHotelera, EntityType, UserRole};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_message};

/// PATCH /api/v1/cadenas-hoteleras/mi-cadena/interfaz - Actualizar solo interfaz de mi cadena
#[instrument(skip(state, auth, request))]
pub async fn patch_mi_cadena_interfaz(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateCadenaInterfazRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mi_cadena = state.container.cadena_hotelera_service
        .get_mi_cadena(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    let old_cadena = state.container.cadena_hotelera_repository
        .find_by_id(mi_cadena.id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Cadena hotelera no encontrada".to_string()))?;
    
    let updated = request.apply_to(old_cadena.clone(), Some(auth.user.id));
    let result = state.container.cadena_hotelera_repository.update(&updated).await?;
    
    let _ = state.container.logging_service.log_update::<CadenaHotelera>(
        Some(auth.user.id),
        Some(auth.user.username.clone()),
        EntityType::CadenaHotelera,
        mi_cadena.id,
        Some(&old_cadena),
        Some(&result),
        Some(vec!["paleta_colores".to_string(), "media".to_string()]),
        None,
    ).await;
    
    Ok(json_ok(CadenaHoteleraResponse::from(result)))
}

/// PATCH /api/v1/cadenas-hoteleras/:id/restore - Restaurar cadena hotelera desactivada
#[instrument(skip(state, auth))]
pub async fn restore_cadena(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para restaurar cadenas hoteleras".to_string()));
    }
    
    let _response = state.container.cadena_hotelera_service
        .restore_cadena(id, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_message("Cadena hotelera restaurada correctamente"))
}
