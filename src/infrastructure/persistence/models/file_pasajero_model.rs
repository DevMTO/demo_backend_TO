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
    pub id_persona: Option<i32>,  // Ahora nullable - pasajeros pueden no tener persona registrada
    pub asiento: Option<String>,
    pub tipo_pasajero: Option<String>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub nacionalidad: Option<String>,
    pub edad: Option<i32>,
    /// Estado: reservado, confirmado, no_show, cancelado
    pub status: String,
}

/// Modelo con datos de persona incluidos (para queries con JOIN)
/// Nota: id_persona ahora puede ser NULL, así que el JOIN debe ser LEFT
#[derive(Debug, Clone, QueryableByName)]
pub struct FilePasajeroWithPersonaModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub id_persona: Option<i32>,  // Ahora nullable
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
    #[diesel(sql_type = Nullable<Integer>)]
    pub edad: Option<i32>,
    /// Estado: reservado, confirmado, no_show, cancelado
    #[diesel(sql_type = Text)]
    pub status: String,
    // Datos de persona (LEFT JOIN = pueden ser NULL)
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
    pub id_persona: Option<i32>,  // Ahora nullable
    pub asiento: Option<&'a str>,
    pub tipo_pasajero: Option<&'a str>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
    pub nacionalidad: Option<&'a str>,
    pub edad: Option<i32>,
    /// Estado: reservado (default), confirmado, no_show, cancelado
    pub status: Option<&'a str>,
}

/// Modelo para actualizar file_pasajeros
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = file_pasajeros)]
pub struct UpdateFilePasajeroModel {
    pub id_persona: Option<Option<i32>>,  // Option<Option> para poder setear a NULL
    pub asiento: Option<String>,
    pub tipo_pasajero: Option<String>,
    pub notas: Option<String>,
    pub nacionalidad: Option<String>,
    pub edad: Option<i32>,
    pub status: Option<String>,
}
