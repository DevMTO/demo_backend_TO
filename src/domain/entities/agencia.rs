//! # Agencia Entity
//! 
//! Entidad para agencias de turismo (tour operators).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Paleta de colores de la agencia (branding)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaletaColores {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    pub background: Option<String>,
    pub text: Option<String>,
}

/// Media/recursos de la agencia
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenciaMedia {
    pub logo: Option<String>,
    pub banner: Option<String>,
    pub images: Vec<String>,
}

/// Entidad Agencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agencia {
    pub id: Uuid,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub paleta_colores: JsonValue,
    pub media: JsonValue,
    pub encargado: Option<Uuid>,  // FK a personas
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agencia {
    pub fn new(nombre: String, ruc: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            nombre,
            ruc,
            telefono: None,
            correo: None,
            direccion: None,
            paleta_colores: serde_json::json!({}),
            media: serde_json::json!({}),
            encargado: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Obtiene la paleta de colores tipada
    pub fn get_paleta(&self) -> Option<PaletaColores> {
        serde_json::from_value(self.paleta_colores.clone()).ok()
    }
    
    /// Obtiene los media tipados
    pub fn get_media(&self) -> Option<AgenciaMedia> {
        serde_json::from_value(self.media.clone()).ok()
    }
}
