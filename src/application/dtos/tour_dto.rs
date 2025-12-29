use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Tour;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TourResponse {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: String,
    pub lugar_fin: String,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub max_personas: Option<i32>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
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
            hora_inicio: t.hora_inicio,
            hora_fin: t.hora_fin,
            detalles: t.detalles,
            itinerario: t.itinerario,
            precio_base: t.precio_base,
            duracion_dias: t.duracion_dias,
            max_personas: t.max_personas,
            media: t.media,
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
    
    #[validate(length(min = 2, max = 200))]
    pub lugar_inicio: String,
    
    #[validate(length(min = 2, max = 200))]
    pub lugar_fin: String,
    
    pub hora_inicio: Option<NaiveTime>,
    
    pub hora_fin: Option<NaiveTime>,
    
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    
    #[validate(range(min = 0.0, message = "Precio debe ser positivo"))]
    pub precio_base: f64,
    
    #[validate(range(min = 1, message = "Duración mínima 1 día"))]
    pub duracion_dias: Option<i32>,
    
    #[validate(range(min = 1, message = "Máximo personas mínimo 1"))]
    pub max_personas: Option<i32>,
    
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
}

impl CreateTourRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Tour {
        let now = Utc::now();
        Tour {
            id: 0,
            nombre: self.nombre,
            lugar_inicio: self.lugar_inicio,
            lugar_fin: self.lugar_fin,
            hora_inicio: self.hora_inicio,
            hora_fin: self.hora_fin,
            detalles: self.detalles,
            itinerario: self.itinerario,
            precio_base: BigDecimal::try_from(self.precio_base).unwrap_or_default(),
            duracion_dias: self.duracion_dias,
            max_personas: self.max_personas,
            media: self.media,
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
    
    #[validate(length(min = 2, max = 200))]
    pub lugar_inicio: Option<String>,
    
    #[validate(length(min = 2, max = 200))]
    pub lugar_fin: Option<String>,
    
    pub hora_inicio: Option<NaiveTime>,
    
    pub hora_fin: Option<NaiveTime>,
    
    #[ts(type = "object | null")]
    pub detalles: Option<JsonValue>,
    
    #[ts(type = "object | null")]
    pub itinerario: Option<JsonValue>,
    
    #[validate(range(min = 0.0))]
    pub precio_base: Option<f64>,
    
    #[validate(range(min = 1))]
    pub duracion_dias: Option<i32>,
    
    #[validate(range(min = 1))]
    pub max_personas: Option<i32>,
    
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    
    pub is_active: Option<bool>,
}

impl UpdateTourRequest {
    pub fn apply_to(self, mut tour: Tour, updated_by: Option<i32>) -> Tour {
        if let Some(nombre) = self.nombre {
            tour.nombre = nombre;
        }
        if let Some(lugar_inicio) = self.lugar_inicio {
            tour.lugar_inicio = lugar_inicio;
        }
        if let Some(lugar_fin) = self.lugar_fin {
            tour.lugar_fin = lugar_fin;
        }
        if let Some(hora_inicio) = self.hora_inicio {
            tour.hora_inicio = Some(hora_inicio);
        }
        if let Some(hora_fin) = self.hora_fin {
            tour.hora_fin = Some(hora_fin);
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
        if let Some(max) = self.max_personas {
            tour.max_personas = Some(max);
        }
        if let Some(media) = self.media {
            tour.media = Some(media);
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
