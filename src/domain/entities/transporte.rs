//! # Transporte Entity
//! 
//! Entidad para empresas de transporte.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entidad Transporte (empresa de transporte)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transporte {
    pub id: Uuid,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub encargado: Option<Uuid>,  // FK a personas
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transporte {
    pub fn new(nombre: String, ruc: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            nombre,
            ruc,
            telefono: None,
            correo: None,
            direccion: None,
            encargado: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
