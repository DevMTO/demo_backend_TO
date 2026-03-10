use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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
    // Nuevos campos movidos desde files
    pub turno_tour: Option<String>,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    /// Estado del file_tour: reservado, confirmado, en_progreso, completado, cancelado
    pub status: String,
    /// Coordenadas de geolocalización del punto de recojo
    pub geo_recojo: Option<JsonValue>,
    /// Cantidad de pasajeros específicos para este tour (null = todos los del file)
    pub nro_pasajeros: Option<i32>,
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
    // Nuevos campos movidos desde files
    pub turno_tour: Option<&'a str>,
    pub lugar_recojo: Option<&'a str>,
    pub hora_recojo: Option<NaiveTime>,
    /// Estado del file_tour (default: reservado)
    pub status: &'a str,
    /// Coordenadas de geolocalización del punto de recojo
    pub geo_recojo: Option<JsonValue>,
    /// Cantidad de pasajeros específicos para este tour
    pub nro_pasajeros: Option<i32>,
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
    // Nuevos campos movidos desde files
    pub turno_tour: Option<Option<&'a str>>,
    pub lugar_recojo: Option<Option<&'a str>>,
    pub hora_recojo: Option<Option<NaiveTime>>,
    /// Estado del file_tour
    pub status: Option<&'a str>,
    /// Coordenadas de geolocalización del punto de recojo
    pub geo_recojo: Option<Option<JsonValue>>,
    /// Cantidad de pasajeros específicos para este tour
    pub nro_pasajeros: Option<Option<i32>>,
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
    // Nuevos campos movidos desde files
    pub turno_tour: Option<String>,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    /// Estado del file_tour
    pub status: String,
    /// Coordenadas de geolocalización del punto de recojo
    pub geo_recojo: Option<JsonValue>,
    /// Cantidad de pasajeros específicos para este tour
    pub nro_pasajeros: Option<i32>,
    // Campos del tour (JOIN)
    pub tour_nombre: String,
    pub tour_lugar_inicio: Option<String>,
    pub tour_lugar_fin: Option<String>,
    pub tour_precio_base: BigDecimal,
    pub tour_duracion_dias: Option<i32>,
    pub tour_tipo: Option<String>,
    pub tour_is_active: bool,
    /// Campos de geolocalización del tour
    pub tour_geo_inicio: Option<JsonValue>,
    pub tour_geo_fin: Option<JsonValue>,
    pub tour_geo_ruta: Option<JsonValue>,
}
