use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::infrastructure::persistence::schema::file_tours;

/// Modelo para la tabla file_tours (relación N:M entre files y tours)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_tours)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileTourModel {
    pub id: i32,
    pub id_file: i32,
    pub id_tour: i32,
    pub orden: i32,
    pub precio_aplicado: Option<BigDecimal>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub fecha_tour: Option<NaiveDate>,
}

/// Modelo para insertar nuevos registros en file_tours
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_tours)]
pub struct NewFileTourModel<'a> {
    pub id_file: i32,
    pub id_tour: i32,
    pub orden: i32,
    pub precio_aplicado: Option<BigDecimal>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
    pub fecha_tour: Option<NaiveDate>,
}

/// Modelo para actualizar registros en file_tours
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = file_tours)]
pub struct UpdateFileTourModel<'a> {
    pub id_tour: Option<i32>,
    pub orden: Option<i32>,
    pub precio_aplicado: Option<Option<BigDecimal>>,
    pub notas: Option<Option<&'a str>>,
    pub fecha_tour: Option<Option<NaiveDate>>,
}

/// Modelo para el resultado del JOIN entre file_tours y tours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTourWithTourModel {
    // Campos de file_tours
    pub id: i32,
    pub id_file: i32,
    pub id_tour: i32,
    pub orden: i32,
    pub precio_aplicado: Option<BigDecimal>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub fecha_tour: Option<NaiveDate>,
    // Campos del tour (JOIN)
    pub tour_nombre: String,
    pub tour_lugar_inicio: String,
    pub tour_lugar_fin: String,
    pub tour_precio_base: BigDecimal,
    pub tour_duracion_dias: Option<i32>,
    pub tour_tipo: Option<String>,
    pub tour_is_active: bool,
}
