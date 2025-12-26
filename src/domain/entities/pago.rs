//! # Pago Entity
//! 
//! Entidad para pagos y movimientos financieros.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Tipo de movimiento
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

/// Evidencia de pago
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenciaPago {
    pub comprobante_url: Option<String>,
    pub tipo: Option<String>,  // "boleta", "factura"
    pub numero: Option<String>,
}

/// Entidad Pago
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pago {
    pub id: Uuid,
    pub id_file: Uuid,
    pub tipo_movimiento: TipoMovimiento,
    pub concepto: String,
    pub monto: f64,
    pub metodo_pago: Option<String>,
    pub referencia: Option<String>,
    pub evidencia: JsonValue,
    pub fecha_pago: DateTime<Utc>,
    pub notas: Option<String>,
    pub registrado_por: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pago {
    pub fn new(id_file: Uuid, tipo_movimiento: TipoMovimiento, concepto: String, monto: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            id_file,
            tipo_movimiento,
            concepto,
            monto,
            metodo_pago: None,
            referencia: None,
            evidencia: serde_json::json!({}),
            fecha_pago: now,
            notas: None,
            registrado_por: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene la evidencia tipada
    pub fn get_evidencia(&self) -> Option<EvidenciaPago> {
        serde_json::from_value(self.evidencia.clone()).ok()
    }
    
    /// Verifica si es un ingreso
    pub fn es_ingreso(&self) -> bool {
        matches!(self.tipo_movimiento, TipoMovimiento::Ingreso | TipoMovimiento::Adelanto)
    }
    
    /// Verifica si es un egreso
    pub fn es_egreso(&self) -> bool {
        matches!(self.tipo_movimiento, TipoMovimiento::Egreso | TipoMovimiento::Reembolso)
    }
    
    /// Monto con signo (positivo ingreso, negativo egreso)
    pub fn monto_con_signo(&self) -> f64 {
        if self.es_egreso() {
            -self.monto
        } else {
            self.monto
        }
    }
    
    /// Monto formateado
    pub fn monto_formateado(&self) -> String {
        let signo = if self.es_egreso() { "-" } else { "+" };
        format!("{} S/ {:.2}", signo, self.monto)
    }
}
