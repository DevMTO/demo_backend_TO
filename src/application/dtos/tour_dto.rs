use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Tour;
use super::geo_dto::{GeoLocation, GeoRoutePoint};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TourResponse {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: Option<String>,
    pub lugar_fin: Option<String>,
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    #[ts(type = "string")]
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    pub tipo_tour: Option<String>,
    /// Horarios del tour: { "full": {...}, "morning": {...}, "afternoon": {...} }
    #[ts(type = "object | null")]
    pub horarios: Option<JsonValue>,
    /// Indica si el tour incluye restaurante en su itinerario
    pub tiene_restaurante: bool,
    /// Geolocalización del punto de inicio del tour
    pub geo_inicio: Option<GeoLocation>,
    /// Geolocalización del punto de fin del tour
    pub geo_fin: Option<GeoLocation>,
    /// Ruta del tour como array de puntos
    pub geo_ruta: Option<Vec<GeoRoutePoint>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tour> for TourResponse {
    fn from(t: Tour) -> Self {
        Self {
            id: t.id,
            nombre: t.nombre,
            lugar_inicio: t.lugar_inicio,
            lugar_fin: t.lugar_fin,
            detalles: t.detalles,
            itinerario: t.itinerario,
            precio_base: t.precio_base,
            duracion_dias: t.duracion_dias,
            media: t.media,
            tipo_tour: t.tipo_tour,
            horarios: t.horarios,
            tiene_restaurante: t.tiene_restaurante,
            geo_inicio: t.geo_inicio.and_then(|v| serde_json::from_value(v).ok()),
            geo_fin: t.geo_fin.and_then(|v| serde_json::from_value(v).ok()),
            geo_ruta: t.geo_ruta.and_then(|v| serde_json::from_value(v).ok()),
            is_active: t.is_active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateTourRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    #[validate(length(max = 200))]
    pub lugar_inicio: Option<String>,
    
    #[validate(length(max = 200))]
    pub lugar_fin: Option<String>,
    
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    
    #[validate(range(min = 0.0, message = "Precio debe ser positivo"))]
    pub precio_base: f64,
    
    #[validate(range(min = 1, message = "Duración mínima 1 día"))]
    pub duracion_dias: Option<i32>,
    
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    
    #[validate(length(max = 100))]
    pub tipo_tour: Option<String>,
    
    /// Horarios del tour: { "full": {"start": "HH:MM", "end": "HH:MM"}, "morning": {...}, "afternoon": {...} }
    #[ts(type = "object | null")]
    pub horarios: Option<JsonValue>,
    
    /// Indica si el tour incluye restaurante en su itinerario
    pub tiene_restaurante: Option<bool>,
    
    /// Geolocalización del punto de inicio del tour
    pub geo_inicio: Option<GeoLocation>,
    
    /// Geolocalización del punto de fin del tour
    pub geo_fin: Option<GeoLocation>,
    
    /// Ruta del tour como array de puntos
    pub geo_ruta: Option<Vec<GeoRoutePoint>>,
}

impl CreateTourRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Tour {
        let now = Utc::now();
        Tour {
            id: 0,
            nombre: self.nombre,
            lugar_inicio: self.lugar_inicio,
            lugar_fin: self.lugar_fin,
            detalles: self.detalles,
            itinerario: self.itinerario,
            precio_base: BigDecimal::try_from(self.precio_base).unwrap_or_default(),
            duracion_dias: self.duracion_dias,
            media: self.media,
            tipo_tour: self.tipo_tour,
            horarios: self.horarios,
            tiene_restaurante: self.tiene_restaurante.unwrap_or(false),
            geo_inicio: self.geo_inicio.map(|g| serde_json::to_value(g).unwrap_or_default()),
            geo_fin: self.geo_fin.map(|g| serde_json::to_value(g).unwrap_or_default()),
            geo_ruta: self.geo_ruta.map(|r| serde_json::to_value(r).unwrap_or_default()),
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateTourRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    #[validate(length(max = 200))]
    pub lugar_inicio: Option<String>,
    
    #[validate(length(max = 200))]
    pub lugar_fin: Option<String>,
    
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    
    #[validate(range(min = 0.0))]
    pub precio_base: Option<f64>,
    
    #[validate(range(min = 1))]
    pub duracion_dias: Option<i32>,
    
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    
    #[validate(length(max = 100))]
    pub tipo_tour: Option<String>,
    
    /// Horarios del tour: { "full": {"start": "HH:MM", "end": "HH:MM"}, "morning": {...}, "afternoon": {...} }
    #[ts(type = "object | null")]
    pub horarios: Option<JsonValue>,
    
    /// Indica si el tour incluye restaurante en su itinerario
    pub tiene_restaurante: Option<bool>,
    
    /// Geolocalización del punto de inicio del tour
    pub geo_inicio: Option<GeoLocation>,
    
    /// Geolocalización del punto de fin del tour
    pub geo_fin: Option<GeoLocation>,
    
    /// Ruta del tour como array de puntos
    pub geo_ruta: Option<Vec<GeoRoutePoint>>,
    
    pub is_active: Option<bool>,
}

impl UpdateTourRequest {
    pub fn apply_to(self, mut tour: Tour, updated_by: Option<i32>) -> Tour {
        if let Some(nombre) = self.nombre {
            tour.nombre = nombre;
        }
        if let Some(lugar_inicio) = self.lugar_inicio {
            tour.lugar_inicio = Some(lugar_inicio);
        }
        if let Some(lugar_fin) = self.lugar_fin {
            tour.lugar_fin = Some(lugar_fin);
        }
        if let Some(detalles) = self.detalles {
            tour.detalles = Some(detalles);
        }
        if let Some(itinerario) = self.itinerario {
            tour.itinerario = Some(itinerario);
        }
        if let Some(precio) = self.precio_base {
            tour.precio_base = BigDecimal::try_from(precio).unwrap_or_default();
        }
        if let Some(duracion) = self.duracion_dias {
            tour.duracion_dias = Some(duracion);
        }
        if let Some(media) = self.media {
            tour.media = Some(media);
        }
        if let Some(tipo_tour) = self.tipo_tour {
            tour.tipo_tour = Some(tipo_tour);
        }
        if let Some(horarios) = self.horarios {
            tour.horarios = Some(horarios);
        }
        if let Some(tiene_restaurante) = self.tiene_restaurante {
            tour.tiene_restaurante = tiene_restaurante;
        }
        // Geolocalización - convertir GeoLocation a JsonValue
        if let Some(geo_inicio) = self.geo_inicio {
            tour.geo_inicio = if geo_inicio.has_data() {
                Some(serde_json::to_value(geo_inicio).unwrap_or_default())
            } else {
                None
            };
        }
        if let Some(geo_fin) = self.geo_fin {
            tour.geo_fin = if geo_fin.has_data() {
                Some(serde_json::to_value(geo_fin).unwrap_or_default())
            } else {
                None
            };
        }
        if let Some(geo_ruta) = self.geo_ruta {
            tour.geo_ruta = if !geo_ruta.is_empty() {
                Some(serde_json::to_value(geo_ruta).unwrap_or_default())
            } else {
                None
            };
        }
        if let Some(is_active) = self.is_active {
            tour.is_active = is_active;
        }
        tour.updated_by = updated_by;
        tour.updated_at = Utc::now();
        tour
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TourListResponse {
    pub items: Vec<TourResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
