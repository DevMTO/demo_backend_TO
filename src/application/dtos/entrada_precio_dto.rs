use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::EntradaPrecio;

/// Response DTO para EntradaPrecio
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct EntradaPrecioResponse {
    pub id: i32,
    pub id_entrada: i32,
    pub tipo_precio: String,
    pub edad_minima: i32,
    pub edad_maxima: Option<i32>,
    #[ts(type = "string")]
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub rango_label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EntradaPrecio> for EntradaPrecioResponse {
    fn from(e: EntradaPrecio) -> Self {
        let rango_label = e.rango_label();
        Self {
            id: e.id,
            id_entrada: e.id_entrada,
            tipo_precio: e.tipo_precio,
            edad_minima: e.edad_minima,
            edad_maxima: e.edad_maxima,
            precio: e.precio,
            descripcion: e.descripcion,
            rango_label,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Request para crear un precio de entrada
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateEntradaPrecioRequest {
    pub id_entrada: i32,
    
    #[validate(length(min = 2, max = 30, message = "Tipo de precio debe tener entre 2 y 30 caracteres"))]
    pub tipo_precio: String,
    
    #[validate(range(min = 0, message = "Edad mínima debe ser positiva"))]
    pub edad_minima: i32,
    
    /// None significa sin límite superior (ej: 17+)
    pub edad_maxima: Option<i32>,
    
    #[validate(range(min = 0.0, message = "Precio debe ser positivo o cero"))]
    pub precio: f64,
    
    #[validate(length(max = 100))]
    pub descripcion: Option<String>,
}

impl CreateEntradaPrecioRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> EntradaPrecio {
        let now = Utc::now();
        EntradaPrecio {
            id: 0,
            id_entrada: self.id_entrada,
            tipo_precio: self.tipo_precio,
            edad_minima: self.edad_minima,
            edad_maxima: self.edad_maxima,
            precio: BigDecimal::try_from(self.precio).unwrap_or_default(),
            descripcion: self.descripcion,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

/// Request para actualizar un precio de entrada
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateEntradaPrecioRequest {
    #[validate(length(min = 2, max = 30))]
    pub tipo_precio: Option<String>,
    
    #[validate(range(min = 0))]
    pub edad_minima: Option<i32>,
    
    pub edad_maxima: Option<Option<i32>>,
    
    #[validate(range(min = 0.0))]
    pub precio: Option<f64>,
    
    #[validate(length(max = 100))]
    pub descripcion: Option<String>,
}

impl UpdateEntradaPrecioRequest {
    pub fn apply_to(self, mut precio: EntradaPrecio, updated_by: Option<i32>) -> EntradaPrecio {
        if let Some(tipo_precio) = self.tipo_precio {
            precio.tipo_precio = tipo_precio;
        }
        if let Some(edad_minima) = self.edad_minima {
            precio.edad_minima = edad_minima;
        }
        if let Some(edad_maxima) = self.edad_maxima {
            precio.edad_maxima = edad_maxima;
        }
        if let Some(precio_val) = self.precio {
            precio.precio = BigDecimal::try_from(precio_val).unwrap_or_default();
        }
        if let Some(descripcion) = self.descripcion {
            precio.descripcion = Some(descripcion);
        }
        precio.updated_by = updated_by;
        precio.updated_at = Utc::now();
        precio
    }
}

/// Request para crear múltiples precios de entrada en batch
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct BatchCreateEntradaPreciosRequest {
    pub id_entrada: i32,
    
    #[validate(length(min = 1, message = "Debe incluir al menos un precio"))]
    pub precios: Vec<PrecioRangoInput>,
}

/// Input para un rango de precio individual
#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PrecioRangoInput {
    #[validate(length(min = 2, max = 30))]
    pub tipo_precio: String,
    
    #[validate(range(min = 0))]
    pub edad_minima: i32,
    
    pub edad_maxima: Option<i32>,
    
    #[validate(range(min = 0.0))]
    pub precio: f64,
    
    #[validate(length(max = 100))]
    pub descripcion: Option<String>,
}
