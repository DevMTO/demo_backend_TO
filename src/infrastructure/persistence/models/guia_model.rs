use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::entities::{Guia, StatusGuia};
use crate::infrastructure::persistence::schema::guias;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = guias)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GuiaModel {
    pub id: i32,
    pub id_persona: i32,
    pub nro_carnet: String,
    pub idiomas: Option<JsonValue>,
    pub especialidades: Option<JsonValue>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = guias)]
pub struct NewGuiaModel<'a> {
    pub id_persona: i32,
    pub nro_carnet: &'a str,
    pub idiomas: Option<JsonValue>,
    pub especialidades: Option<JsonValue>,
    pub status: &'a str,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = guias)]
pub struct UpdateGuiaModel<'a> {
    pub nro_carnet: Option<&'a str>,
    pub idiomas: Option<Option<JsonValue>>,
    pub especialidades: Option<Option<JsonValue>>,
    pub status: Option<&'a str>,
    pub updated_by: Option<i32>,
    pub is_active: Option<bool>,
}

impl From<GuiaModel> for Guia {
    fn from(model: GuiaModel) -> Self {
        Guia {
            id: model.id,
            id_persona: model.id_persona,
            nro_carnet: model.nro_carnet,
            idiomas: model.idiomas,
            especialidades: model.especialidades,
            status: model.status.parse().unwrap_or_default(),
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Guia> for NewGuiaModel<'a> {
    fn from(g: &'a Guia) -> Self {
        NewGuiaModel {
            id_persona: g.id_persona,
            nro_carnet: &g.nro_carnet,
            idiomas: g.idiomas.clone(),
            especialidades: g.especialidades.clone(),
            status: match &g.status {
                StatusGuia::Disponible => "disponible",
                StatusGuia::EnServicio => "en_servicio",
                StatusGuia::Inactivo => "inactivo",
                StatusGuia::Suspendido => "suspendido",
            },
            created_by: g.created_by,
            updated_by: g.updated_by,
            is_active: g.is_active,
        }
    }
}
