use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Tarifa;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TarifaResponse {
    pub id: i32,
    pub id_tour: i32,
    pub tipo_entidad: String,
    #[ts(type = "string")]
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tarifa> for TarifaResponse {
    fn from(t: Tarifa) -> Self {
        Self {
            id: t.id,
            id_tour: t.id_tour,
            tipo_entidad: t.tipo_entidad,
            precio: t.precio,
            descripcion: t.descripcion,
            is_active: t.is_active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateTarifaRequest {
    pub id_tour: i32,

    #[validate(length(min = 1, max = 50, message = "tipo_entidad debe tener entre 1 y 50 caracteres"))]
    pub tipo_entidad: String,

    #[validate(range(min = 0.0, message = "Precio debe ser positivo"))]
    pub precio: f64,

    pub descripcion: Option<String>,
}

impl CreateTarifaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Tarifa {
        let now = Utc::now();
        Tarifa {
            id: 0,
            id_tour: self.id_tour,
            tipo_entidad: self.tipo_entidad,
            precio: BigDecimal::try_from(self.precio).unwrap_or_default(),
            descripcion: self.descripcion,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateTarifaRequest {
    #[validate(length(min = 1, max = 50))]
    pub tipo_entidad: Option<String>,

    #[validate(range(min = 0.0))]
    pub precio: Option<f64>,

    pub descripcion: Option<String>,

    pub is_active: Option<bool>,
}

impl UpdateTarifaRequest {
    pub fn apply_to(self, mut tarifa: Tarifa, updated_by: Option<i32>) -> Tarifa {
        if let Some(tipo_entidad) = self.tipo_entidad {
            tarifa.tipo_entidad = tipo_entidad;
        }
        if let Some(precio) = self.precio {
            tarifa.precio = BigDecimal::try_from(precio).unwrap_or_default();
        }
        if let Some(descripcion) = self.descripcion {
            tarifa.descripcion = Some(descripcion);
        }
        if let Some(is_active) = self.is_active {
            tarifa.is_active = is_active;
        }
        tarifa.updated_by = updated_by;
        tarifa.updated_at = Utc::now();
        tarifa
    }
}
