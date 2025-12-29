use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;
use validator::Validate;

use crate::domain::entities::{Pago, TipoMovimiento};

#[derive(Debug, Clone, Serialize)]
pub struct PagoResponse {
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
}

impl From<Pago> for PagoResponse {
    fn from(p: Pago) -> Self {
        Self {
            id: p.id,
            id_file: p.id_file,
            tipo_movimiento: p.tipo_movimiento,
            concepto: p.concepto,
            monto: p.monto,
            metodo_pago: p.metodo_pago,
            referencia: p.referencia,
            evidencia: p.evidencia,
            fecha_pago: p.fecha_pago,
            notas: p.notas,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePagoRequest {
    pub id_file: i32,
    
    #[validate(length(min = 2, max = 30, message = "Tipo de movimiento inválido"))]
    pub tipo_movimiento: String,
    
    #[validate(length(min = 2, max = 200, message = "Concepto debe tener entre 2 y 200 caracteres"))]
    pub concepto: String,
    
    #[validate(range(min = 0.01, message = "Monto debe ser positivo"))]
    pub monto: f64,
    
    #[validate(length(max = 50))]
    pub metodo_pago: Option<String>,
    
    #[validate(length(max = 100))]
    pub referencia: Option<String>,
    
    /// Evidencia del pago (comprobante, etc.)
    pub evidencia: Option<JsonValue>,
    
    pub fecha_pago: Option<DateTime<Utc>>,
    
    pub notas: Option<String>,
}

impl CreatePagoRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Pago {
        let now = Utc::now();
        let tipo = self.tipo_movimiento.parse::<TipoMovimiento>().unwrap_or_default();
        
        Pago {
            id: 0,
            id_file: self.id_file,
            tipo_movimiento: tipo.to_string(),
            concepto: self.concepto,
            monto: BigDecimal::try_from(self.monto).unwrap_or_default(),
            metodo_pago: self.metodo_pago,
            referencia: self.referencia,
            evidencia: self.evidencia,
            fecha_pago: self.fecha_pago.unwrap_or(now),
            notas: self.notas,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePagoRequest {
    #[validate(length(min = 2, max = 30))]
    pub tipo_movimiento: Option<String>,
    
    #[validate(length(min = 2, max = 200))]
    pub concepto: Option<String>,
    
    #[validate(range(min = 0.01))]
    pub monto: Option<f64>,
    
    #[validate(length(max = 50))]
    pub metodo_pago: Option<String>,
    
    #[validate(length(max = 100))]
    pub referencia: Option<String>,
    
    pub evidencia: Option<JsonValue>,
    
    pub fecha_pago: Option<DateTime<Utc>>,
    
    pub notas: Option<String>,
}

impl UpdatePagoRequest {
    pub fn apply_to(self, mut pago: Pago, updated_by: Option<i32>) -> Pago {
        if let Some(tipo) = self.tipo_movimiento {
            pago.tipo_movimiento = tipo;
        }
        if let Some(concepto) = self.concepto {
            pago.concepto = concepto;
        }
        if let Some(monto) = self.monto {
            pago.monto = BigDecimal::try_from(monto).unwrap_or_default();
        }
        if let Some(metodo) = self.metodo_pago {
            pago.metodo_pago = Some(metodo);
        }
        if let Some(referencia) = self.referencia {
            pago.referencia = Some(referencia);
        }
        if let Some(evidencia) = self.evidencia {
            pago.evidencia = Some(evidencia);
        }
        if let Some(fecha) = self.fecha_pago {
            pago.fecha_pago = fecha;
        }
        if let Some(notas) = self.notas {
            pago.notas = Some(notas);
        }
        pago.updated_by = updated_by;
        pago.updated_at = Utc::now();
        pago
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PagoListResponse {
    pub items: Vec<PagoResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileBalanceResponse {
    pub id_file: i32,
    pub total_ingresos: BigDecimal,
    pub total_egresos: BigDecimal,
    pub balance: BigDecimal,
    pub movimientos_count: i64,
}
