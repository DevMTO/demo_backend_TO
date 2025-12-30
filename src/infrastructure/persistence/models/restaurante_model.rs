use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

use crate::domain::entities::Restaurante;
use crate::infrastructure::persistence::schema::restaurantes;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = restaurantes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RestauranteModel {
    pub id: i32,
    pub nombre: String,
    pub direccion: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub tipo_atencion: Option<JsonValue>,
    pub precio_promedio: Option<BigDecimal>,
    pub capacidad: Option<i32>,
    pub horario: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub encargado: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = restaurantes)]
pub struct NewRestauranteModel<'a> {
    pub nombre: &'a str,
    pub direccion: &'a str,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub tipo_atencion: Option<JsonValue>,
    pub precio_promedio: Option<BigDecimal>,
    pub capacidad: Option<i32>,
    pub horario: Option<JsonValue>,
    pub is_active: bool,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub encargado: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = restaurantes)]
pub struct UpdateRestauranteModel<'a> {
    pub nombre: Option<&'a str>,
    pub direccion: Option<&'a str>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub tipo_atencion: Option<Option<JsonValue>>,
    pub precio_promedio: Option<Option<BigDecimal>>,
    pub capacidad: Option<Option<i32>>,
    pub horario: Option<Option<JsonValue>>,
    pub is_active: Option<bool>,
    pub updated_by: Option<i32>,
    pub encargado: Option<Option<i32>>,
}

impl From<RestauranteModel> for Restaurante {
    fn from(model: RestauranteModel) -> Self {
        Restaurante {
            id: model.id,
            nombre: model.nombre,
            direccion: model.direccion,
            telefono: model.telefono,
            correo: model.correo,
            tipo_atencion: model.tipo_atencion,
            precio_promedio: model.precio_promedio,
            capacidad: model.capacidad,
            horario: model.horario,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
            encargado: model.encargado,
        }
    }
}

impl<'a> From<&'a Restaurante> for NewRestauranteModel<'a> {
    fn from(r: &'a Restaurante) -> Self {
        NewRestauranteModel {
            nombre: &r.nombre,
            direccion: &r.direccion,
            telefono: r.telefono.as_deref(),
            correo: r.correo.as_deref(),
            tipo_atencion: r.tipo_atencion.clone(),
            precio_promedio: r.precio_promedio.clone(),
            capacidad: r.capacidad,
            horario: r.horario.clone(),
            is_active: r.is_active,
            created_by: r.created_by,
            updated_by: r.updated_by,
            encargado: r.encargado,
        }
    }
}
