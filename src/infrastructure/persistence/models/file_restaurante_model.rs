use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::infrastructure::persistence::schema::file_restaurantes;

/// Modelo de Diesel para file_restaurantes (vinculado a file_tours)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_restaurantes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileRestauranteModel {
    pub id: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub precio: Option<BigDecimal>,
    pub id_file_tour: i32,
    /// Estado: reservado, confirmado, cancelado
    pub status: String,
}

/// Modelo insertable para crear file_restaurantes
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_restaurantes)]
pub struct NewFileRestauranteModel<'a> {
    pub id_file_tour: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<&'a str>,
    pub created_by: Option<i32>,
    pub precio: Option<BigDecimal>,
    /// Estado: reservado (default), confirmado, cancelado
    pub status: Option<&'a str>,
}
