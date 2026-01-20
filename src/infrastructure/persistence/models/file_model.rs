use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use crate::domain::entities::File;
use crate::infrastructure::persistence::schema::files;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileModel {
    pub id: i32,
    pub id_agencia: i32,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    pub notas: Option<String>,
    pub status: String,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
    pub nro_pasajeros: i32,
    pub file_code: Option<String>,
    pub turno_tour: Option<String>,
    pub deadline_confirmacion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = files)]
pub struct NewFileModel<'a> {
    pub id_agencia: i32,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub lugar_recojo: Option<&'a str>,
    pub hora_recojo: Option<NaiveTime>,
    pub notas: Option<&'a str>,
    pub status: &'a str,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
    pub nro_pasajeros: i32,
    pub file_code: Option<&'a str>,
    pub turno_tour: Option<&'a str>,
    pub deadline_confirmacion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = files)]
pub struct UpdateFileModel<'a> {
    pub id_agencia: Option<i32>,
    pub fecha_inicio: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,
    pub lugar_recojo: Option<Option<&'a str>>,
    pub hora_recojo: Option<Option<NaiveTime>>,
    pub notas: Option<Option<&'a str>>,
    pub status: Option<&'a str>,
    pub monto_total: Option<BigDecimal>,
    pub monto_pagado: Option<BigDecimal>,
    pub updated_by: Option<i32>,
    pub is_active: Option<bool>,
    pub nro_pasajeros: Option<i32>,
    pub file_code: Option<Option<&'a str>>,
    pub turno_tour: Option<Option<&'a str>>,
    pub deadline_confirmacion: Option<Option<DateTime<Utc>>>,
}

impl From<FileModel> for File {
    fn from(model: FileModel) -> Self {
        File {
            id: model.id,
            id_agencia: model.id_agencia,
            fecha_inicio: model.fecha_inicio,
            fecha_fin: model.fecha_fin,
            lugar_recojo: model.lugar_recojo,
            hora_recojo: model.hora_recojo,
            notas: model.notas,
            status: model.status,
            monto_total: model.monto_total,
            monto_pagado: model.monto_pagado,
            is_active: model.is_active,
            nro_pasajeros: model.nro_pasajeros,
            file_code: model.file_code,
            turno_tour: model.turno_tour,
            deadline_confirmacion: model.deadline_confirmacion,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a File> for NewFileModel<'a> {
    fn from(f: &'a File) -> Self {
        NewFileModel {
            id_agencia: f.id_agencia,
            fecha_inicio: f.fecha_inicio,
            fecha_fin: f.fecha_fin,
            lugar_recojo: f.lugar_recojo.as_deref(),
            hora_recojo: f.hora_recojo,
            notas: f.notas.as_deref(),
            status: &f.status,
            monto_total: f.monto_total.clone(),
            monto_pagado: f.monto_pagado.clone(),
            created_by: f.created_by,
            updated_by: f.updated_by,
            is_active: f.is_active,
            nro_pasajeros: f.nro_pasajeros,
            file_code: f.file_code.as_deref(),
            turno_tour: f.turno_tour.as_deref(),
            deadline_confirmacion: f.deadline_confirmacion,
        }
    }
}
