use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::domain::entities::Tarifa;
use crate::infrastructure::persistence::schema::tarifas;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = tarifas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TarifaModel {
    pub id: i32,
    pub id_tour: i32,
    pub tipo_entidad: String,
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tarifas)]
pub struct NewTarifaModel<'a> {
    pub id_tour: i32,
    pub tipo_entidad: &'a str,
    pub precio: BigDecimal,
    pub descripcion: Option<&'a str>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = tarifas)]
pub struct UpdateTarifaModel<'a> {
    pub tipo_entidad: Option<&'a str>,
    pub precio: Option<BigDecimal>,
    pub descripcion: Option<Option<&'a str>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<TarifaModel> for Tarifa {
    fn from(model: TarifaModel) -> Self {
        Tarifa {
            id: model.id,
            id_tour: model.id_tour,
            tipo_entidad: model.tipo_entidad,
            precio: model.precio,
            descripcion: model.descripcion,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Tarifa> for NewTarifaModel<'a> {
    fn from(t: &'a Tarifa) -> Self {
        NewTarifaModel {
            id_tour: t.id_tour,
            tipo_entidad: &t.tipo_entidad,
            precio: t.precio.clone(),
            descripcion: t.descripcion.as_deref(),
            created_by: t.created_by,
            updated_by: t.updated_by,
        }
    }
}
