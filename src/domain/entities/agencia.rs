use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaletaColores {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    pub background: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgenciaMedia {
    pub logo: Option<String>,
    pub banner: Option<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agencia {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub paleta_colores: Option<JsonValue>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,  // FK a personas
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Agencia {
    pub fn new(nombre: String, ruc: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            nombre,
            ruc,
            telefono: None,
            correo: None,
            direccion: None,
            paleta_colores: Some(serde_json::json!({})),
            media: Some(serde_json::json!({})),
            encargado: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Obtiene la paleta de colores tipada
    pub fn get_paleta(&self) -> Option<PaletaColores> {
        self.paleta_colores.as_ref().and_then(|p| serde_json::from_value(p.clone()).ok())
    }
    
    /// Obtiene los media tipados
    pub fn get_media(&self) -> Option<AgenciaMedia> {
        self.media.as_ref().and_then(|m| serde_json::from_value(m.clone()).ok())
    }
}
