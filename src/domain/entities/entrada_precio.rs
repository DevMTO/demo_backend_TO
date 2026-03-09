use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// EntradaPrecio - Distribución de precio por entrada según tipo y rango de edad
/// 
/// Estructura:
/// - tipo_precio: general, nacional, extranjero
/// - Rangos de edad:
///   * 0-8 años: generalmente gratis (precio = 0)
///   * 9-16 años: precio de niño/adolescente
///   * 17+ años (edad_maxima = None): precio de adulto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaPrecio {
    pub id: i32,
    pub id_entrada: i32,
    pub tipo_precio: String, // Stored as varchar: 'general', 'nacional', 'extranjero'
    pub edad_minima: i32,
    pub edad_maxima: Option<i32>, // None = sin límite superior (ej: 17+)
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl EntradaPrecio {
    pub fn new(
        id_entrada: i32,
        tipo_precio: String,
        edad_minima: i32,
        edad_maxima: Option<i32>,
        precio: BigDecimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_entrada,
            tipo_precio,
            edad_minima,
            edad_maxima,
            precio,
            descripcion: None,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }

    /// Verifica si una edad cae dentro de este rango
    pub fn matches_edad(&self, edad: i32) -> bool {
        if edad < self.edad_minima {
            return false;
        }
        match self.edad_maxima {
            Some(max) => edad <= max,
            None => true, // Sin límite superior
        }
    }

    /// Obtiene el label del rango de edad
    pub fn rango_label(&self) -> String {
        match self.edad_maxima {
            Some(max) => format!("{}-{} años", self.edad_minima, max),
            None => format!("{}+ años", self.edad_minima),
        }
    }
}
