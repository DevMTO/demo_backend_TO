use chrono::{DateTime, Utc};
use diesel::prelude::*;
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
}

/// Modelo insertable para crear file_vehiculos
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_vehiculos)]
pub struct NewFileVehiculoModel {
    pub id_file_tour: i32,
    pub id_vehiculo: i32,
    pub id_conductor: Option<i32>,
    pub created_by: Option<i32>,
    pub capacidad_asignada: i32,
}
