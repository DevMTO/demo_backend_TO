use chrono::{DateTime, NaiveTime, Utc};
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
    pub lugar_inicio: String,
    pub lugar_fin: String,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub max_personas: Option<i32>,
    pub media: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tours)]
pub struct NewTourModel<'a> {
    pub nombre: &'a str,
    pub lugar_inicio: &'a str,
    pub lugar_fin: &'a str,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub detalles: Option<JsonValue>,
    pub itinerario: Option<JsonValue>,
    pub precio_base: BigDecimal,
    pub duracion_dias: Option<i32>,
    pub max_personas: Option<i32>,
    pub media: Option<JsonValue>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = tours)]
pub struct UpdateTourModel<'a> {
    pub nombre: Option<&'a str>,
    pub lugar_inicio: Option<&'a str>,
    pub lugar_fin: Option<&'a str>,
    pub hora_inicio: Option<Option<NaiveTime>>,
    pub hora_fin: Option<Option<NaiveTime>>,
    pub detalles: Option<Option<JsonValue>>,
    pub itinerario: Option<Option<JsonValue>>,
    pub precio_base: Option<BigDecimal>,
    pub duracion_dias: Option<Option<i32>>,
    pub max_personas: Option<Option<i32>>,
    pub media: Option<Option<JsonValue>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<TourModel> for Tour {
    fn from(model: TourModel) -> Self {
        Tour {
            id: model.id,
            nombre: model.nombre,
            lugar_inicio: model.lugar_inicio,
            lugar_fin: model.lugar_fin,
            hora_inicio: model.hora_inicio,
            hora_fin: model.hora_fin,
            detalles: model.detalles,
            itinerario: model.itinerario,
            precio_base: model.precio_base,
            duracion_dias: model.duracion_dias,
            max_personas: model.max_personas,
            media: model.media,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Tour> for NewTourModel<'a> {
    fn from(tour: &'a Tour) -> Self {
        NewTourModel {
            nombre: &tour.nombre,
            lugar_inicio: &tour.lugar_inicio,
            lugar_fin: &tour.lugar_fin,
            hora_inicio: tour.hora_inicio,
            hora_fin: tour.hora_fin,
            detalles: tour.detalles.clone(),
            itinerario: tour.itinerario.clone(),
            precio_base: tour.precio_base.clone(),
            duracion_dias: tour.duracion_dias,
            max_personas: tour.max_personas,
            media: tour.media.clone(),
            is_active: tour.is_active,
            created_by: tour.created_by,
            updated_by: tour.updated_by,
        }
    }
}
