use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_vehiculos;

/// Modelo de Diesel para file_vehiculos
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_vehiculos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileVehiculoModel {
    pub id: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub capacidad_asignada: i32,
    pub id_file_tour: i32,
    /// Estado: reservado, confirmado, cancelado
    pub status: String,
}

/// Modelo con datos completos de vehículo, conductor, transporte y persona del conductor
/// JOIN: file_vehiculos -> vehiculos -> transportes
///       file_vehiculos -> conductores -> personas
#[derive(Debug, Clone, QueryableByName)]
pub struct FileVehiculoWithPersonaModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file_tour: i32,
    #[diesel(sql_type = Integer)]
    pub id_vehiculo: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub id_conductor: Option<i32>,
    #[diesel(sql_type = Integer)]
    pub capacidad_asignada: i32,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub created_by: Option<i32>,
    #[diesel(sql_type = Text)]
    pub status: String,
    // Datos del vehículo
    #[diesel(sql_type = Nullable<Text>)]
    pub vehiculo_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub vehiculo_placa: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub vehiculo_capacidad: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub vehiculo_modelo: Option<String>,
    // Datos del transporte (empresa dueña del vehículo)
    #[diesel(sql_type = Nullable<Integer>)]
    pub transporte_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub transporte_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub transporte_ruc: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub transporte_telefono: Option<String>,
    // Datos del conductor
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_brevete: Option<String>,
    // Datos de la persona del conductor
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_apellidos: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_telefono: Option<String>,
}

/// Modelo insertable para crear file_vehiculos
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_vehiculos)]
pub struct NewFileVehiculoModel<'a> {
    pub id_file_tour: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub created_by: Option<i32>,
    pub capacidad_asignada: i32,
    /// Estado: reservado (default), confirmado, cancelado
    pub status: Option<&'a str>,
}

/// Modelo para actualizar file_vehiculos (PATCH parcial)
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = file_vehiculos)]
pub struct UpdateFileVehiculoModel {
    pub id_vehiculo: Option<i32>,
    pub id_conductor: Option<Option<i32>>,  // Option<Option> para poder setear a NULL
    pub capacidad_asignada: Option<i32>,
    pub status: Option<String>,
}

