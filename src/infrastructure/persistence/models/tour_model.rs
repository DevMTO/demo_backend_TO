use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

use crate::domain::entities::Tour;
use crate::infrastructure::persistence::schema::tours;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tours)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TourModel {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: Option<String>,
    pub lugar_fin: Option<String>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub media: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub tipo_tour: Option<String>,
    pub horarios: Option<JsonValue>,
    pub tiene_restaurante: bool,
    pub geo_inicio: Option<JsonValue>,
    pub geo_fin: Option<JsonValue>,
    pub geo_ruta: Option<JsonValue>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tours)]
pub struct NewTourModel<'a> {
    pub nombre: &'a str,
    pub lugar_inicio: Option<&'a str>,
    pub lugar_fin: Option<&'a str>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub media: Option<JsonValue>,
    pub tipo_tour: Option<&'a str>,
    pub horarios: Option<JsonValue>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub tiene_restaurante: Option<bool>,
    pub geo_inicio: Option<JsonValue>,
    pub geo_fin: Option<JsonValue>,
    pub geo_ruta: Option<JsonValue>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = tours)]
pub struct UpdateTourModel<'a> {
    pub nombre: Option<&'a str>,
    pub lugar_inicio: Option<Option<&'a str>>,
    pub lugar_fin: Option<Option<&'a str>>,
    pub detalles: Option<Option<JsonValue>>,
    pub itinerario: Option<Option<JsonValue>>,
    pub precio_base: Option<BigDecimal>,
    pub duracion_dias: Option<Option<i32>>,
    pub media: Option<Option<JsonValue>>,
    pub tipo_tour: Option<Option<&'a str>>,
    pub horarios: Option<Option<JsonValue>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
    pub tiene_restaurante: Option<bool>,
    pub geo_inicio: Option<Option<JsonValue>>,
    pub geo_fin: Option<Option<JsonValue>>,
    pub geo_ruta: Option<Option<JsonValue>>,
}

impl From<TourModel> for Tour {
    fn from(model: TourModel) -> Self {
        Tour {
            id: model.id,
            nombre: model.nombre,
            lugar_inicio: model.lugar_inicio,
            lugar_fin: model.lugar_fin,
            detalles: model.detalles,
            itinerario: model.itinerario,
            precio_base: model.precio_base,
            duracion_dias: model.duracion_dias,
            media: model.media,
            tipo_tour: model.tipo_tour,
            horarios: model.horarios,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
            tiene_restaurante: model.tiene_restaurante,
            geo_inicio: model.geo_inicio,
            geo_fin: model.geo_fin,
            geo_ruta: model.geo_ruta,
        }
    }
}

impl<'a> From<&'a Tour> for NewTourModel<'a> {
    fn from(tour: &'a Tour) -> Self {
        NewTourModel {
            nombre: &tour.nombre,
            lugar_inicio: tour.lugar_inicio.as_deref(),
            lugar_fin: tour.lugar_fin.as_deref(),
            detalles: tour.detalles.clone(),
            itinerario: tour.itinerario.clone(),
            precio_base: tour.precio_base.clone(),
            duracion_dias: tour.duracion_dias,
            media: tour.media.clone(),
            tipo_tour: tour.tipo_tour.as_deref(),
            horarios: tour.horarios.clone(),
            is_active: tour.is_active,
            created_by: tour.created_by,
            updated_by: tour.updated_by,
            tiene_restaurante: Some(tour.tiene_restaurante),
            geo_inicio: tour.geo_inicio.clone(),
            geo_fin: tour.geo_fin.clone(),
            geo_ruta: tour.geo_ruta.clone(),
        }
    }
}
