use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// Entidad FileTour - Representa la relación N:M entre Files y Tours
/// Permite que un File tenga múltiples tours asignados con orden y precios específicos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTour {
    pub id: i32,
    pub id_file: i32,
    pub id_tour: i32,
    /// Orden del tour dentro del file (1, 2, 3...)
    pub orden: i32,
    /// Precio aplicado para este tour en este file (puede diferir del precio_base)
    pub precio_aplicado: Option<BigDecimal>,
    /// Notas específicas para este tour en el contexto del file
    pub notas: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

impl FileTour {
    pub fn new(id_file: i32, id_tour: i32, orden: i32) -> Self {
        Self {
            id: 0,
            id_file,
            id_tour,
            orden,
            precio_aplicado: None,
            notas: None,
            created_at: Utc::now(),
            created_by: None,
        }
    }
    
    pub fn with_precio(mut self, precio: BigDecimal) -> Self {
        self.precio_aplicado = Some(precio);
        self
    }
    
    pub fn with_notas(mut self, notas: String) -> Self {
        self.notas = Some(notas);
        self
    }
}
