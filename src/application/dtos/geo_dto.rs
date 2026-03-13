//! DTOs para geolocalización
//!
//! Estructura unificada para todos los campos de geolocalización (geo_inicio, geo_fin, geo_recojo, etc.)

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Fuente de la geolocalización
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum GeoSource {
    /// Seleccionado en el mapa
    Map,
    /// Escrito manualmente
    Typed,
    /// Obtenido por GPS del dispositivo
    Gps,
}

/// Estructura unificada para geolocalización
/// Compatible con JSONB de PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GeoLocation {
    /// Latitud (coordenada Y)
    #[ts(optional)]
    pub lat: Option<f64>,

    /// Longitud (coordenada X)
    #[ts(optional)]
    pub lng: Option<f64>,

    /// Dirección o nombre del lugar legible para humanos
    #[ts(optional)]
    pub address: Option<String>,

    /// Referencia adicional ("frente a...", "al lado de...")
    #[ts(optional)]
    pub reference: Option<String>,

    /// ID del lugar en proveedores externos (Google Places, Mapbox, etc.)
    #[ts(optional)]
    pub place_id: Option<String>,

    /// Fuente de la geolocalización: 'map' | 'typed' | 'gps'
    #[ts(optional)]
    pub source: Option<GeoSource>,

    /// Fecha de última actualización (ISO 8601)
    #[ts(optional)]
    pub updated_at: Option<String>,

    /// Nivel de zoom del mapa (para restaurar la vista)
    #[ts(optional)]
    pub zoom: Option<i32>,
}

impl GeoLocation {
    /// Crea un nuevo GeoLocation vacío
    pub fn new() -> Self {
        Self {
            lat: None,
            lng: None,
            address: None,
            reference: None,
            place_id: None,
            source: None,
            updated_at: None,
            zoom: None,
        }
    }

    /// Verifica si tiene coordenadas válidas
    pub fn has_coords(&self) -> bool {
        self.lat.is_some() && self.lng.is_some()
    }

    /// Verifica si tiene algún dato útil
    pub fn has_data(&self) -> bool {
        self.has_coords()
            || self
                .address
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
            || self
                .reference
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
    }
}

impl Default for GeoLocation {
    fn default() -> Self {
        Self::new()
    }
}

/// Punto de una ruta (solo coordenadas para arrays de puntos)
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct GeoRoutePoint {
    pub lat: f64,
    pub lng: f64,
}
