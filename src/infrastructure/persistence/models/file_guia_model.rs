use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::schema::file_guias;

/// Modelo de Diesel para file_guias
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = file_guias)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileGuiaModel {
    pub id: i32,
    pub id_guia: i32,
    pub rol: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub estado_confirmacion: String,
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<String>,
    pub id_file_tour: i32,
    /// Estado: pendiente (si no aceptó), reservado (si aceptó), confirmado, cancelado
    pub status: String,
}

/// Modelo con datos de persona incluidos (para queries con JOIN)
/// JOIN: file_guias -> guias -> personas
#[derive(Debug, Clone, QueryableByName)]
pub struct FileGuiaWithPersonaModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file_tour: i32,
    #[diesel(sql_type = Integer)]
    pub id_guia: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub rol: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub created_by: Option<i32>,
    #[diesel(sql_type = Text)]
    pub estado_confirmacion: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub confirmado_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    pub motivo_rechazo: Option<String>,
    #[diesel(sql_type = Text)]
    pub status: String,
    // Datos del guía (de tabla guias)
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_nro_carnet: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_idiomas: Option<String>,
    // Datos de la persona asociada al guía (de tabla personas)
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_apellidos: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_telefono: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub guia_correo: Option<String>,
}

/// Modelo insertable para crear file_guias
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_guias)]
pub struct NewFileGuiaModel<'a> {
    pub id_file_tour: i32,
    pub id_guia: i32,
    pub rol: Option<&'a str>,
    pub created_by: Option<i32>,
    /// Auto-aceptado al asignar (igual que conductor)
    pub estado_confirmacion: Option<&'a str>,
    /// Estado: reservado (auto-asignado)
    pub status: Option<&'a str>,
}

/// Modelo para actualizar estado de confirmación
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = file_guias)]
pub struct UpdateFileGuiaConfirmacionModel<'a> {
    pub estado_confirmacion: &'a str,
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<&'a str>,
}

/// Modelo para actualizar file_guias (PATCH parcial: id_guia, id_file_tour)
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = file_guias)]
pub struct UpdateFileGuiaModel {
    pub id_guia: Option<i32>,
    pub id_file_tour: Option<i32>,
}

