//! Handlers para FileEntradas (entradas vinculadas a file_tours)

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{AssignEntradaToFileTourRequest, FileEntradaResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};

/// Lista las entradas asignadas a un file_tour
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_entradas(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el file_tour existe
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    let entradas = state.container.file_entrada_repository
        .find_by_file_tour(file_tour_id)
        .await?;
    
    // Obtener información completa de cada entrada
    let mut responses: Vec<FileEntradaResponse> = Vec::new();
    for e in entradas {
        let mut response = FileEntradaResponse::from(e.clone());
        if let Ok(Some(entrada)) = state.container.entrada_repository.find_by_id(e.id_entrada).await {
            response.entrada_nombre = Some(entrada.nombre);
            // El precio ahora se obtiene de entrada_precios
            response.entrada_precio = None;
        }
        responses.push(response);
    }
    
    Ok(json_ok(responses))
}

/// Asigna una entrada a un file_tour
#[instrument(skip(state, auth, request))]
pub async fn assign_entrada_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignEntradaToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe
    state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    // Verificar que la entrada existe
    state.container.entrada_repository
        .find_by_id(request.id_entrada)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Entrada {} no encontrada", request.id_entrada)))?;
    
    let result = state.container.file_entrada_repository
        .add(request.id_file_tour, request.id_entrada, request.cantidad, request.id_entrada_precio, Some(auth.user.id))
        .await?;
    
    Ok(json_created(FileEntradaResponse::from(result)))
}

/// Elimina una asignación de entrada
#[instrument(skip(state, _auth))]
pub async fn remove_file_entrada(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(entrada_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que existe la asignación
    state.container.file_entrada_repository
        .find_by_id(entrada_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    state.container.file_entrada_repository.remove(entrada_asig_id).await?;
    Ok(json_deleted())
}
