use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::domain::entities::Entrada;
use crate::infrastructure::persistence::schema::entradas;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = entradas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EntradaModel {
    pub id: i32,
    pub nombre: String,
    pub precio: BigDecimal,
    pub ruta: Option<String>,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = entradas)]
pub struct NewEntradaModel<'a> {
    pub nombre: &'a str,
    pub precio: BigDecimal,
    pub ruta: Option<&'a str>,
    pub tipo: &'a str,
    pub descripcion: Option<&'a str>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = entradas)]
pub struct UpdateEntradaModel<'a> {
    pub nombre: Option<&'a str>,
    pub precio: Option<BigDecimal>,
    pub ruta: Option<Option<&'a str>>,
    pub tipo: Option<&'a str>,
    pub descripcion: Option<Option<&'a str>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
}

impl From<EntradaModel> for Entrada {
    fn from(model: EntradaModel) -> Self {
        Entrada {
            id: model.id,
            nombre: model.nombre,
            precio: model.precio,
            ruta: model.ruta,
            tipo: model.tipo,
            descripcion: model.descripcion,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Entrada> for NewEntradaModel<'a> {
    fn from(e: &'a Entrada) -> Self {
        NewEntradaModel {
            nombre: &e.nombre,
            precio: e.precio.clone(),
            ruta: e.ruta.as_deref(),
            tipo: &e.tipo,
            descripcion: e.descripcion.as_deref(),
            is_active: e.is_active,
            created_by: e.created_by,
            updated_by: e.updated_by,
        }
    }
}
