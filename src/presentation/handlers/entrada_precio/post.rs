//! POST handlers para EntradaPrecio

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{CreateEntradaPrecioRequest, BatchCreateEntradaPreciosRequest, EntradaPrecioResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_created;

/// POST /api/entrada-precios
/// Crear un nuevo precio de entrada
#[instrument(skip(state, auth, request))]
pub async fn create_precio(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateEntradaPrecioRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let entity = request.into_entity(Some(auth.user.id));
    let created = state.container.entrada_precio_service
        .create_precio(&entity)
        .await?;
    
    Ok(json_created(EntradaPrecioResponse::from(created)))
}

/// POST /api/entradas/:id_entrada/precios/batch
/// Crear múltiples precios de entrada en batch
#[instrument(skip(state, auth, request))]
pub async fn create_precios_batch(
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
        .create_precios_batch(&entities)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = created
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_created(response))
}

/// POST /api/entradas/:id_entrada/precios/initialize
/// Inicializar precios por defecto para una entrada
#[instrument(skip(state, auth))]
pub async fn initialize_default_precios(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id_entrada): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let created = state.container.entrada_precio_service
        .initialize_default_precios(id_entrada, Some(auth.user.id))
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = created
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_created(response))
}
