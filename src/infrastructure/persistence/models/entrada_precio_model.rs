use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::domain::entities::EntradaPrecio;
use crate::infrastructure::persistence::schema::entrada_precios;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = entrada_precios)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EntradaPrecioModel {
    pub id: i32,
    pub id_entrada: i32,
    pub tipo_precio: String,
    pub edad_minima: i32,
    pub edad_maxima: Option<i32>,
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = entrada_precios)]
pub struct NewEntradaPrecioModel<'a> {
    pub id_entrada: i32,
    pub tipo_precio: &'a str,
    pub edad_minima: i32,
    pub edad_maxima: Option<i32>,
    pub precio: BigDecimal,
    pub descripcion: Option<&'a str>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = entrada_precios)]
pub struct UpdateEntradaPrecioModel<'a> {
    pub tipo_precio: Option<&'a str>,
    pub edad_minima: Option<i32>,
    pub edad_maxima: Option<Option<i32>>,
    pub precio: Option<BigDecimal>,
    pub descripcion: Option<Option<&'a str>>,
    pub updated_by: Option<i32>,
}

impl From<EntradaPrecioModel> for EntradaPrecio {
    fn from(model: EntradaPrecioModel) -> Self {
        EntradaPrecio {
            id: model.id,
            id_entrada: model.id_entrada,
            tipo_precio: model.tipo_precio,
            edad_minima: model.edad_minima,
            edad_maxima: model.edad_maxima,
            precio: model.precio,
            descripcion: model.descripcion,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a EntradaPrecio> for NewEntradaPrecioModel<'a> {
    fn from(e: &'a EntradaPrecio) -> Self {
        NewEntradaPrecioModel {
            id_entrada: e.id_entrada,
            tipo_precio: &e.tipo_precio,
            edad_minima: e.edad_minima,
            edad_maxima: e.edad_maxima,
            precio: e.precio.clone(),
            descripcion: e.descripcion.as_deref(),
            created_by: e.created_by,
            updated_by: e.updated_by,
        }
    }
}
