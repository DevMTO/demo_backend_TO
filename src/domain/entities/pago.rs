
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoMovimiento {
    Ingreso,
    Egreso,
    Adelanto,
    Saldo,
    Reembolso,
}

impl std::fmt::Display for TipoMovimiento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoMovimiento::Ingreso => write!(f, "ingreso"),
            TipoMovimiento::Egreso => write!(f, "egreso"),
            TipoMovimiento::Adelanto => write!(f, "adelanto"),
            TipoMovimiento::Saldo => write!(f, "saldo"),
            TipoMovimiento::Reembolso => write!(f, "reembolso"),
        }
    }
}

impl std::str::FromStr for TipoMovimiento {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ingreso" => Ok(TipoMovimiento::Ingreso),
            "egreso" => Ok(TipoMovimiento::Egreso),
            "adelanto" => Ok(TipoMovimiento::Adelanto),
            "saldo" => Ok(TipoMovimiento::Saldo),
            "reembolso" => Ok(TipoMovimiento::Reembolso),
            _ => Err(format!("Tipo de movimiento inválido: {s}")),
        }
    }
}

impl Default for TipoMovimiento {
    fn default() -> Self {
        TipoMovimiento::Ingreso
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenciaPago {
    pub comprobante_url: Option<String>,
    pub tipo: Option<String>,  // "boleta", "factura"
    pub numero: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pago {
    pub id: i32,
    pub id_file: i32,
    pub tipo_movimiento: String, // Stored as varchar in DB
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

impl Pago {
    pub fn new(id_file: i32, tipo_movimiento: TipoMovimiento, concepto: String, monto: BigDecimal) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_file,
            tipo_movimiento: tipo_movimiento.to_string(),
            concepto,
            monto,
            metodo_pago: None,
            referencia: None,
            evidencia: Some(serde_json::json!({})),
            fecha_pago: now,
            notas: None,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Obtiene el tipo como enum
    pub fn get_tipo_movimiento(&self) -> TipoMovimiento {
        self.tipo_movimiento.parse().unwrap_or_default()
    }
    
    /// Obtiene la evidencia tipada
    pub fn get_evidencia(&self) -> Option<EvidenciaPago> {
        self.evidencia.as_ref()
            .and_then(|e| serde_json::from_value(e.clone()).ok())
    }
    
    /// Verifica si es un ingreso
    pub fn es_ingreso(&self) -> bool {
        let tipo = self.get_tipo_movimiento();
        matches!(tipo, TipoMovimiento::Ingreso | TipoMovimiento::Adelanto)
    }
    
    /// Verifica si es un egreso
    pub fn es_egreso(&self) -> bool {
        let tipo = self.get_tipo_movimiento();
        matches!(tipo, TipoMovimiento::Egreso | TipoMovimiento::Reembolso)
    }
}
