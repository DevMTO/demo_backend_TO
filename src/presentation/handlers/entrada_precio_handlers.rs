use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{
    CreateEntradaPrecioRequest, UpdateEntradaPrecioRequest, 
    EntradaPrecioResponse, BatchCreateEntradaPreciosRequest,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use super::common::{json_ok, json_created, json_message};

/// GET /api/entradas/:id_entrada/precios
/// Obtener todos los precios de una entrada
#[instrument(skip(state, _auth))]
pub async fn list_precios_by_entrada(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_entrada): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precios = state.container.entrada_precio_service
        .get_precios_by_entrada(id_entrada)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = precios
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_ok(response))
}

/// GET /api/entradas/:id_entrada/precios/tipo/:tipo_precio
/// Obtener precios de una entrada por tipo (general, nacional, extranjero)
#[instrument(skip(state, _auth))]
pub async fn list_precios_by_tipo(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((id_entrada, tipo_precio)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precios = state.container.entrada_precio_service
        .get_precios_by_tipo(id_entrada, &tipo_precio)
        .await?;
    
    let response: Vec<EntradaPrecioResponse> = precios
        .into_iter()
        .map(EntradaPrecioResponse::from)
        .collect();
    
    Ok(json_ok(response))
}

/// GET /api/entrada-precios/:id
/// Obtener un precio específico por ID
#[instrument(skip(state, _auth))]
pub async fn get_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precio = state.container.entrada_precio_service.get_precio(id).await?;
    Ok(json_ok(EntradaPrecioResponse::from(precio)))
}

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

/// DELETE /api/entrada-precios/:id
/// Eliminar un precio de entrada
#[instrument(skip(state, _auth))]
pub async fn delete_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.entrada_precio_service.delete_precio(id).await?;
    Ok(json_message("Precio eliminado"))
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

/// Calcular precio Query params
#[derive(Debug, serde::Deserialize)]
pub struct CalcularPrecioQuery {
    pub edad: i32,
    pub tipo_turista: String, // "nacional" o "extranjero"
}

/// GET /api/entradas/:id_entrada/calcular-precio?edad=25&tipo_turista=nacional
/// Calcular el precio aplicable para una entrada según edad y tipo de turista
#[instrument(skip(state, _auth))]
pub async fn calcular_precio(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id_entrada): Path<i32>,
    axum::extract::Query(query): axum::extract::Query<CalcularPrecioQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let precio = state.container.entrada_precio_service
        .calcular_precio(id_entrada, query.edad, &query.tipo_turista)
        .await?;
    
    Ok(json_ok(serde_json::json!({
        "id_entrada": id_entrada,
        "edad": query.edad,
        "tipo_turista": query.tipo_turista,
        "precio": precio.to_string()
    })))
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
