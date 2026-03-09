//! PUT handlers para Hoteles

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::UpdateHotelRequest;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/v1/hoteles/:id - Actualizar hotel
#[instrument(skip(state, auth, request))]
pub async fn update_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateHotelRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !matches!(auth.user.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApplicationError::Forbidden("No tienes permisos para actualizar hoteles".to_string()));
    }
    
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.hotel_service
        .update_hotel(id, request, auth.user.id, Some(auth.user.username.clone()))
        .await?;
    
    Ok(json_ok(response))
}

/// PUT /api/v1/hoteles/mi-hotel - Actualizar mi propio hotel (campos limitados)
#[instrument(skip(state, auth, request))]
pub async fn update_mi_hotel(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateHotelRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let mi_hotel = state.container.hotel_service
        .get_mi_hotel(
            &auth.user.role,
            auth.user.id_entidad,
            auth.user.id_persona,
            &auth.user.username,
        )
        .await?;
    
    // Solo permitir ciertos campos
    let limited_request = UpdateHotelRequest {
        id_cadena: None,
        nombre: None,
        categoria: request.categoria,
        telefono: request.telefono,
        correo: request.correo,
        direccion: request.direccion,
        ciudad: request.ciudad,
        media: request.media,
        encargado: None,
        is_active: None,
    };
    
    let response = state.container.hotel_service
        .update_hotel(
            mi_hotel.id,
            limited_request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;
    
    Ok(json_ok(response))
}
