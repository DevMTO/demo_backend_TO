use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tarifa {
    pub id: i32,
    pub id_tour: i32,
    pub tipo_entidad: String,   // "agencias", "hoteles", etc.
    pub precio: BigDecimal,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}
