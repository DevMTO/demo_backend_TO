//! Handlers para FileRestaurantes (restaurantes vinculados a file_tours)

use axum::{extract::{Path, State}, response::IntoResponse, Json};
use tracing::instrument;
use validator::Validate;

use crate::application::dtos::{AssignRestauranteToFileTourRequest, FileRestauranteResponse};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_ok, json_created, json_deleted};

/// Lista los restaurantes asignados a un file_tour
#[instrument(skip(state, _auth))]
pub async fn list_file_tour_restaurantes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_tour_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_tour_repository
        .find_by_id(file_tour_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
    
    let restaurantes = state.container.file_restaurante_repository
        .find_by_file_tour(file_tour_id)
        .await?;
    
    // Obtener información completa de cada restaurante
    let mut responses: Vec<FileRestauranteResponse> = Vec::new();
    for r in restaurantes {
        let mut response = FileRestauranteResponse::from(r.clone());
        if let Ok(Some(restaurante)) = state.container.restaurante_repository.find_by_id(r.id_restaurante).await {
            response.restaurante_nombre = Some(restaurante.nombre);
            response.restaurante_direccion = Some(restaurante.direccion);
        }
        responses.push(response);
    }
    
    Ok(json_ok(responses))
}

/// Asigna un restaurante a un file_tour específico
#[instrument(skip(state, auth, request))]
pub async fn assign_restaurante_to_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AssignRestauranteToFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el file_tour existe
    let file_tour = state.container.file_tour_repository
        .find_by_id(request.id_file_tour)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
    
    let restaurante = state.container.restaurante_repository
        .find_by_id(request.id_restaurante)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("Restaurante {} no encontrado", request.id_restaurante)))?;
    
    let servicio = request.tipo_servicio.clone().unwrap_or_else(|| "Almuerzo".to_string());
    
    // Convertir precio de f64 a BigDecimal si se proporciona
    let precio = request.precio.map(|p| bigdecimal::BigDecimal::try_from(p).unwrap_or_default());
    
    let result = state.container.file_restaurante_repository
        .add(
            request.id_file_tour, 
            request.id_restaurante, 
            request.tipo_servicio.as_deref(),
            precio,
            Some(auth.user.id),
        )
        .await?;
    
    // ===== NOTIFICAR AL RESTAURANTE ASIGNADO =====
    // Obtener información del file para la notificación
    let file = state.container.file_repository
        .find_by_id(file_tour.id_file)
        .await?;
    
    if let Some(file) = file {
        // Obtener información del tour
        let tour = state.container.tour_repository
            .find_by_id(file_tour.id_tour)
            .await?;
        
        let tour_name = tour.map(|t| t.nombre.clone()).unwrap_or_else(|| "Tour".to_string());
        let file_code = file.file_code.clone().unwrap_or_else(|| format!("F-{}", file.id));
        let fecha = file.fecha_inicio.format("%d/%m/%Y").to_string();
        
        // Notificar al restaurante
        let _ = state.container.file_assignment_service
            .notify_restaurante_assignment(
                restaurante.id,
                &file_code,
                &tour_name,
                &fecha,
                &servicio,
                Some(auth.user.id),
            ).await;
    }
    
    Ok(json_created(FileRestauranteResponse::from(result)))
}

/// Elimina una asignación de restaurante
#[instrument(skip(state, _auth))]
pub async fn remove_file_restaurante(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(restaurante_asig_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    state.container.file_restaurante_repository
        .find_by_id(restaurante_asig_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound("Asignación no encontrada".to_string()))?;
    
    state.container.file_restaurante_repository.remove(restaurante_asig_id).await?;
    Ok(json_deleted())
}
