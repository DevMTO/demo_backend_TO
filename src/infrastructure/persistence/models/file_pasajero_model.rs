use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_pasajeros;

/// Modelo de Diesel para file_pasajeros
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_pasajeros)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FilePasajeroModel {
    pub id: i32,
    pub id_file: i32,
    pub id_persona: i32,
    pub asiento: Option<String>,
    pub tipo_pasajero: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

/// Modelo insertable para crear file_pasajeros
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_pasajeros)]
pub struct NewFilePasajeroModel<'a> {
    pub id_file: i32,
    pub id_persona: i32,
    pub asiento: Option<&'a str>,
    pub tipo_pasajero: Option<&'a str>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
}
