use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::Agencia;
use crate::infrastructure::persistence::schema::agencias;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = agencias)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgenciaModel {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub paleta_colores: Option<JsonValue>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = agencias)]
pub struct NewAgenciaModel<'a> {
    pub nombre: &'a str,
    pub ruc: &'a str,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub direccion: Option<&'a str>,
    pub paleta_colores: Option<JsonValue>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = agencias)]
pub struct UpdateAgenciaModel<'a> {
    pub nombre: Option<&'a str>,
    pub ruc: Option<&'a str>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub direccion: Option<Option<&'a str>>,
    pub paleta_colores: Option<Option<JsonValue>>,
    pub media: Option<Option<JsonValue>>,
    pub encargado: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<AgenciaModel> for Agencia {
    fn from(model: AgenciaModel) -> Self {
        Agencia {
            id: model.id,
            nombre: model.nombre,
            ruc: model.ruc,
            telefono: model.telefono,
            correo: model.correo,
            direccion: model.direccion,
            paleta_colores: model.paleta_colores,
            media: model.media,
            encargado: model.encargado,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Agencia> for NewAgenciaModel<'a> {
    fn from(agencia: &'a Agencia) -> Self {
        NewAgenciaModel {
            nombre: &agencia.nombre,
            ruc: &agencia.ruc,
            telefono: agencia.telefono.as_deref(),
            correo: agencia.correo.as_deref(),
            direccion: agencia.direccion.as_deref(),
            paleta_colores: agencia.paleta_colores.clone(),
            media: agencia.media.clone(),
            encargado: agencia.encargado,
            is_active: agencia.is_active,
            created_by: agencia.created_by,
            updated_by: agencia.updated_by,
        }
    }
}
