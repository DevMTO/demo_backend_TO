use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_guias;

/// Modelo de Diesel para file_guias
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_guias)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileGuiaModel {
    pub id: i32,
    pub id_file: i32,
    pub id_guia: i32,
    pub rol: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

/// Modelo insertable para crear file_guias
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_guias)]
pub struct NewFileGuiaModel<'a> {
    pub id_file: i32,
    pub id_guia: i32,
    pub rol: Option<&'a str>,
    pub created_by: Option<i32>,
}
