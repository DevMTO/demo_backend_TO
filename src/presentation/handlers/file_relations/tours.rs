//! Handlers para FileTours (tours vinculados a files)

use axum::{extract::{Path, State}, response::IntoResponse};
use tracing::instrument;

use crate::application::dtos::FileTourDto;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// Lista los tours asignados a un file
#[instrument(skip(state, _auth))]
pub async fn list_file_tours(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(file_id): Path<i32>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el file existe
    state.container.file_repository
        .find_by_id(file_id)
        .await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
    
    // Obtener tours con información completa (INNER JOIN)
    let tours = state.container.file_tour_repository
        .find_by_file_with_tour(file_id)
        .await?;
    
    // Convertir a DTOs
    let responses: Vec<FileTourDto> = tours.into_iter()
        .map(|t| FileTourDto {
            id: t.id,
            id_tour: t.id_tour,
            orden: t.orden,
            precio_aplicado: t.precio_aplicado.clone(),
            notas: t.notas,
            fecha_tour: t.fecha_tour,
            turno_tour: t.turno_tour,
            lugar_recojo: t.lugar_recojo,
            hora_recojo: t.hora_recojo,
            // Convertir JsonValue a GeoLocation
            geo_recojo: t.geo_recojo.and_then(|v| serde_json::from_value(v).ok()),
            status: t.status,
            nro_pasajeros: t.nro_pasajeros,
            tour_nombre: Some(t.tour_nombre),
            tour_lugar_inicio: t.tour_lugar_inicio,
            tour_lugar_fin: t.tour_lugar_fin,
            tour_precio_base: None,
            tour_duracion_dias: t.tour_duracion_dias,
            tour_tipo: t.tour_tipo,
            tour_is_active: Some(t.tour_is_active),
        })
        .collect();
    
    Ok(json_ok(responses))
}
