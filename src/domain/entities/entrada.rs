
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrada {
    pub id: i32,
    pub nombre: String,
    pub ruta: Option<String>,  // Lugar/ruta del ticket
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Entrada {
    pub fn new(nombre: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            ruta: None,
            descripcion: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
}
