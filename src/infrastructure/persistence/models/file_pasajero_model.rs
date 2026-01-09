use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
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
    pub nacionalidad: Option<String>,
}

/// Modelo con datos de persona incluidos (para queries con JOIN)
#[derive(Debug, Clone, QueryableByName)]
pub struct FilePasajeroWithPersonaModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file: i32,
    #[diesel(sql_type = Integer)]
    pub id_persona: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub asiento: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub tipo_pasajero: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub notas: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub created_by: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub nacionalidad: Option<String>,
    // Datos de persona
    #[diesel(sql_type = Nullable<Text>)]
    pub pasajero_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub pasajero_apellidos: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub pasajero_documento: Option<String>,
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
    pub nacionalidad: Option<&'a str>,
}
