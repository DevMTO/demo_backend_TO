use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::Transporte;
use crate::infrastructure::persistence::schema::transportes;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = transportes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransporteModel {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub media: Option<JsonValue>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = transportes)]
pub struct NewTransporteModel<'a> {
    pub nombre: &'a str,
    pub ruc: &'a str,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub direccion: Option<&'a str>,
    pub encargado: Option<i32>,
    pub media: Option<JsonValue>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = transportes)]
pub struct UpdateTransporteModel<'a> {
    pub nombre: Option<&'a str>,
    pub ruc: Option<&'a str>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub direccion: Option<Option<&'a str>>,
    pub encargado: Option<Option<i32>>,
    pub media: Option<Option<JsonValue>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<TransporteModel> for Transporte {
    fn from(model: TransporteModel) -> Self {
        Transporte {
            id: model.id,
            nombre: model.nombre,
            ruc: model.ruc,
            telefono: model.telefono,
            correo: model.correo,
            direccion: model.direccion,
            encargado: model.encargado,
            media: model.media,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Transporte> for NewTransporteModel<'a> {
    fn from(transporte: &'a Transporte) -> Self {
        NewTransporteModel {
            nombre: &transporte.nombre,
            ruc: &transporte.ruc,
            telefono: transporte.telefono.as_deref(),
            correo: transporte.correo.as_deref(),
            direccion: transporte.direccion.as_deref(),
            encargado: transporte.encargado,
            media: transporte.media.clone(),
            is_active: transporte.is_active,
            created_by: transporte.created_by,
            updated_by: transporte.updated_by,
        }
    }
}
