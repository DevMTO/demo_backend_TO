//! PUT handlers para EntradaPrecio

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{UpdateEntradaPrecioRequest, BatchCreateEntradaPreciosRequest, EntradaPrecioResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// PUT /api/entrada-precios/:id
/// Actualizar un precio de entrada
#[instrument(skip(state, auth, request))]
pub async fn update_precio(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(request): Json<UpdateEntradaPrecioRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let old_precio = state.container.entrada_precio_service.get_precio(id).await?;
    let updated = request.apply_to(old_precio, Some(auth.user.id));
    
    let result = state.container.entrada_precio_service
        .update_precio(&updated)
        .await?;
    
    Ok(json_ok(EntradaPrecioResponse::from(result)))
}

/// PUT /api/entradas/:id_entrada/precios/replace
/// Reemplazar todos los precios de una entrada
#[instrument(skip(state, auth, request))]
pub async fn replace_all_precios(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_entrada): Path<i32>,
    Json(request): Json<BatchCreateEntradaPreciosRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let now = chrono::Utc::now();
    let entities: Vec<_> = request.precios.into_iter().map(|p| {
        crate::domain::entities::EntradaPrecio {
            id: 0,
            id_entrada,
            tipo_precio: p.tipo_precio,
            edad_minima: p.edad_minima,
            edad_maxima: p.edad_maxima,
            precio: bigdecimal::BigDecimal::try_from(p.precio).unwrap_or_default(),
            descripcion: p.descripcion,
            created_at: now,
            updated_at: now,
            created_by: Some(auth.user.id),
            updated_by: Some(auth.user.id),
        }
    }).collect();
    
    let created = state.container.entrada_precio_service
        .replace_all_precios(id_entrada, &entities)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = created
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_ok(response))
}
