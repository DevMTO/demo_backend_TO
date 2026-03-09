use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::CadenaHotelera;
use crate::infrastructure::persistence::schema::cadenas_hoteleras;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = cadenas_hoteleras)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CadenaHoteleraModel {
    pub id: i32,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub paleta_colores: Option<JsonValue>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = cadenas_hoteleras)]
pub struct NewCadenaHoteleraModel<'a> {
    pub nombre: &'a str,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub paleta_colores: Option<JsonValue>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = cadenas_hoteleras)]
pub struct UpdateCadenaHoteleraModel<'a> {
    pub nombre: Option<&'a str>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub media: Option<Option<JsonValue>>,
    pub encargado: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
    pub paleta_colores: Option<Option<JsonValue>>,
}

impl From<CadenaHoteleraModel> for CadenaHotelera {
    fn from(model: CadenaHoteleraModel) -> Self {
        CadenaHotelera {
            id: model.id,
            nombre: model.nombre,
            telefono: model.telefono,
            correo: model.correo,
            media: model.media,
            encargado: model.encargado,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
            paleta_colores: model.paleta_colores,
        }
    }
}

impl<'a> From<&'a CadenaHotelera> for NewCadenaHoteleraModel<'a> {
    fn from(c: &'a CadenaHotelera) -> Self {
        NewCadenaHoteleraModel {
            nombre: &c.nombre,
            telefono: c.telefono.as_deref(),
            correo: c.correo.as_deref(),
            media: c.media.clone(),
            encargado: c.encargado,
            is_active: c.is_active,
            created_by: c.created_by,
            updated_by: c.updated_by,
            paleta_colores: c.paleta_colores.clone(),
        }
    }
}
