use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotel {
    pub id: i32,
    pub id_cadena: i32,
    pub nombre: String,
    pub categoria: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub ciudad: Option<String>,
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl Hotel {
    pub fn new(nombre: String, id_cadena: i32) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            id_cadena,
            nombre,
            categoria: None,
            telefono: None,
            correo: None,
            direccion: None,
            ciudad: None,
            media: Some(serde_json::json!({})),
            encargado: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
}
