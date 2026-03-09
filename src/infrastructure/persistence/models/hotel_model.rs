use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::entities::Hotel;
use crate::infrastructure::persistence::schema::hoteles;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = hoteles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HotelModel {
    pub id: i32,
    pub id_cadena: i32,
    pub nombre: String,
    pub categoria: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub ciudad: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = hoteles)]
pub struct NewHotelModel<'a> {
    pub id_cadena: i32,
    pub nombre: &'a str,
    pub categoria: Option<&'a str>,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub direccion: Option<&'a str>,
    pub ciudad: Option<&'a str>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = hoteles)]
pub struct UpdateHotelModel<'a> {
    pub id_cadena: Option<i32>,
    pub nombre: Option<&'a str>,
    pub categoria: Option<Option<&'a str>>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub direccion: Option<Option<&'a str>>,
    pub ciudad: Option<Option<&'a str>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<HotelModel> for Hotel {
    fn from(model: HotelModel) -> Self {
        Hotel {
            id: model.id,
            id_cadena: model.id_cadena,
            nombre: model.nombre,
            categoria: model.categoria,
            telefono: model.telefono,
            correo: model.correo,
            direccion: model.direccion,
            ciudad: model.ciudad,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Hotel> for NewHotelModel<'a> {
    fn from(h: &'a Hotel) -> Self {
        NewHotelModel {
            id_cadena: h.id_cadena,
            nombre: &h.nombre,
            categoria: h.categoria.as_deref(),
            telefono: h.telefono.as_deref(),
            correo: h.correo.as_deref(),
            direccion: h.direccion.as_deref(),
            ciudad: h.ciudad.as_deref(),
            is_active: h.is_active,
            created_by: h.created_by,
            updated_by: h.updated_by,
        }
    }
}
