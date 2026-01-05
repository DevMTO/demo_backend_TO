use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_restaurantes;

/// Modelo de Diesel para file_restaurantes
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_restaurantes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileRestauranteModel {
    pub id: i32,
    pub id_file: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<String>,
    pub dia: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

/// Modelo insertable para crear file_restaurantes
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_restaurantes)]
pub struct NewFileRestauranteModel<'a> {
    pub id_file: i32,
    pub id_restaurante: i32,
    pub tipo_servicio: Option<&'a str>,
    pub dia: Option<i32>,
    pub created_by: Option<i32>,
}
