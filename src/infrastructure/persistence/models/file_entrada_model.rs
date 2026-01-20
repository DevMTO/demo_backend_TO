use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_entradas;

/// Modelo de Diesel para file_entradas (vinculado a file_tours)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_entradas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileEntradaModel {
    pub id: i32,
    pub id_entrada: i32,
    pub cantidad: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub id_file_tour: i32,
}

/// Modelo insertable para crear file_entradas
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_entradas)]
pub struct NewFileEntradaModel {
    pub id_file_tour: i32,
    pub id_entrada: i32,
    pub cantidad: i32,
    pub created_by: Option<i32>,
}
