use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Conductor, StatusConductor};
use crate::infrastructure::persistence::schema::conductores;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = conductores)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConductorModel {
    pub id: i32,
    pub id_persona: i32,
    pub id_transporte: Option<i32>,
    pub nro_brevete: String,
    pub tiene_soat: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = conductores)]
pub struct NewConductorModel<'a> {
    pub id_persona: i32,
    pub id_transporte: Option<i32>,
    pub nro_brevete: &'a str,
    pub tiene_soat: bool,
    pub status: &'a str,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = conductores)]
pub struct UpdateConductorModel<'a> {
    pub id_transporte: Option<Option<i32>>,
    pub nro_brevete: Option<&'a str>,
    pub tiene_soat: Option<bool>,
    pub status: Option<&'a str>,
    pub updated_by: Option<i32>,
    pub is_active: Option<bool>,
}

impl From<ConductorModel> for Conductor {
    fn from(model: ConductorModel) -> Self {
        Conductor {
            id: model.id,
            id_persona: model.id_persona,
            id_transporte: model.id_transporte,
            nro_brevete: model.nro_brevete,
            tiene_soat: model.tiene_soat,
            status: model.status.parse().unwrap_or_default(),
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Conductor> for NewConductorModel<'a> {
    fn from(c: &'a Conductor) -> Self {
        NewConductorModel {
            id_persona: c.id_persona,
            id_transporte: c.id_transporte,
            nro_brevete: &c.nro_brevete,
            tiene_soat: c.tiene_soat,
            status: match &c.status {
                StatusConductor::Disponible => "disponible",
                StatusConductor::EnServicio => "en_servicio",
                StatusConductor::Inactivo => "inactivo",
                StatusConductor::Suspendido => "suspendido",
            },
            created_by: c.created_by,
            updated_by: c.updated_by,
            is_active: c.is_active,
        }
    }
}
