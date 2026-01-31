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
    
    /// Crea un GeoLocation desde coordenadas
    pub fn from_coords(lat: f64, lng: f64) -> Self {
        Self {
            lat: Some(lat),
            lng: Some(lng),
            address: None,
            reference: None,
            place_id: None,
            source: Some(GeoSource::Map),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            zoom: None,
        }
    }
    
    /// Crea un GeoLocation desde una dirección de texto
    pub fn from_address(address: String) -> Self {
        Self {
            lat: None,
            lng: None,
            address: Some(address),
            reference: None,
            place_id: None,
            source: Some(GeoSource::Typed),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
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
            || self.address.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
            || self.reference.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
    }
    
    /// Convierte a texto legible
    pub fn to_text(&self) -> Option<String> {
        let addr = self.address.as_ref().filter(|s| !s.trim().is_empty());
        let reference = self.reference.as_ref().filter(|s| !s.trim().is_empty());
        
        let main = if let Some(a) = addr {
            Some(a.clone())
        } else if self.has_coords() {
            Some(format!("{:.6}, {:.6}", self.lat.unwrap(), self.lng.unwrap()))
        } else {
            None
        };
        
        match (main, reference) {
            (Some(m), Some(r)) => Some(format!("{} ({})", m, r)),
            (Some(m), None) => Some(m),
            (None, Some(r)) => Some(r.clone()),
            (None, None) => None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_geo_location_from_coords() {
        let geo = GeoLocation::from_coords(-12.0464, -77.0428);
        assert!(geo.has_coords());
        assert!(geo.has_data());
        assert_eq!(geo.source, Some(GeoSource::Map));
    }
    
    #[test]
    fn test_geo_location_from_address() {
        let geo = GeoLocation::from_address("Hotel Marriott Lima".to_string());
        assert!(!geo.has_coords());
        assert!(geo.has_data());
        assert_eq!(geo.source, Some(GeoSource::Typed));
    }
    
    #[test]
    fn test_geo_location_to_text() {
        let geo = GeoLocation {
            lat: Some(-12.0464),
            lng: Some(-77.0428),
            address: Some("Plaza de Armas".to_string()),
            reference: Some("frente a la catedral".to_string()),
            ..Default::default()
        };
        assert_eq!(geo.to_text(), Some("Plaza de Armas (frente a la catedral)".to_string()));
    }
}
