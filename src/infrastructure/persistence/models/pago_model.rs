use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

use crate::domain::entities::Pago;
use crate::infrastructure::persistence::schema::pagos;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = pagos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PagoModel {
    pub id: i32,
    pub id_file: i32,
    pub tipo_movimiento: String,
    pub concepto: String,
    pub monto: BigDecimal,
    pub metodo_pago: Option<String>,
    pub referencia: Option<String>,
    pub evidencia: Option<JsonValue>,
    pub fecha_pago: DateTime<Utc>,
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = pagos)]
pub struct NewPagoModel<'a> {
    pub id_file: i32,
    pub tipo_movimiento: &'a str,
    pub concepto: &'a str,
    pub monto: BigDecimal,
    pub metodo_pago: Option<&'a str>,
    pub referencia: Option<&'a str>,
    pub evidencia: Option<JsonValue>,
    pub fecha_pago: DateTime<Utc>,
    pub notas: Option<&'a str>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = pagos)]
pub struct UpdatePagoModel<'a> {
    pub tipo_movimiento: Option<&'a str>,
    pub concepto: Option<&'a str>,
    pub monto: Option<BigDecimal>,
    pub metodo_pago: Option<Option<&'a str>>,
    pub referencia: Option<Option<&'a str>>,
    pub evidencia: Option<Option<JsonValue>>,
    pub fecha_pago: Option<DateTime<Utc>>,
    pub notas: Option<Option<&'a str>>,
    pub updated_by: Option<i32>,
}

impl From<PagoModel> for Pago {
    fn from(model: PagoModel) -> Self {
        Pago {
            id: model.id,
            id_file: model.id_file,
            tipo_movimiento: model.tipo_movimiento,
            concepto: model.concepto,
            monto: model.monto,
            metodo_pago: model.metodo_pago,
            referencia: model.referencia,
            evidencia: model.evidencia,
            fecha_pago: model.fecha_pago,
            notas: model.notas,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Pago> for NewPagoModel<'a> {
    fn from(p: &'a Pago) -> Self {
        NewPagoModel {
            id_file: p.id_file,
            tipo_movimiento: &p.tipo_movimiento,
            concepto: &p.concepto,
            monto: p.monto.clone(),
            metodo_pago: p.metodo_pago.as_deref(),
            referencia: p.referencia.as_deref(),
            evidencia: p.evidencia.clone(),
            fecha_pago: p.fecha_pago,
            notas: p.notas.as_deref(),
            created_by: p.created_by,
            updated_by: p.updated_by,
        }
    }
}
