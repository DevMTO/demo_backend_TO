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
    pub estado_confirmacion: String,
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<String>,
}

/// Modelo insertable para crear file_guias
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = file_guias)]
pub struct NewFileGuiaModel<'a> {
    pub id_file: i32,
    pub id_guia: i32,
    pub rol: Option<&'a str>,
    pub created_by: Option<i32>,
    // estado_confirmacion usa DEFAULT 'pendiente' en la DB
}

/// Modelo para actualizar estado de confirmación
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = file_guias)]
pub struct UpdateFileGuiaConfirmacionModel<'a> {
    pub estado_confirmacion: &'a str,
    pub confirmado_at: Option<DateTime<Utc>>,
    pub motivo_rechazo: Option<&'a str>,
}

