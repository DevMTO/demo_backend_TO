use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::{Guia, StatusGuia};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct GuiaResponse {
    pub id: i32,
    pub id_persona: i32,
    pub nro_carnet: String,
    #[ts(type = "object | null")]
    pub idiomas: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub especialidades: Option<JsonValue>,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Guia> for GuiaResponse {
    fn from(g: Guia) -> Self {
        Self {
            id: g.id,
            id_persona: g.id_persona,
            nro_carnet: g.nro_carnet,
            idiomas: g.idiomas,
            especialidades: g.especialidades,
            status: g.status.to_string(), // Enum → String
            is_active: g.is_active,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateGuiaRequest {
    pub id_persona: i32,
    
    #[validate(length(min = 6, max = 30, message = "Número de carnet inválido"))]
    pub nro_carnet: String,
    
    /// Array de idiomas ["español", "inglés", "francés"]
    pub idiomas: Option<Vec<String>>,
    
    /// Array de especialidades ["city tour", "aventura", "histórico"]
    pub especialidades: Option<Vec<String>>,
}

impl CreateGuiaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Guia {
        let now = Utc::now();
        Guia {
            id: 0,
            id_persona: self.id_persona,
            nro_carnet: self.nro_carnet,
            idiomas: self.idiomas.map(|i| serde_json::json!(i)),
            especialidades: self.especialidades.map(|e| serde_json::json!(e)),
            status: StatusGuia::Disponible,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateGuiaRequest {
    pub id_persona: Option<i32>,
    
    #[validate(length(min = 6, max = 30))]
    pub nro_carnet: Option<String>,
    
    pub idiomas: Option<Vec<String>>,
    
    pub especialidades: Option<Vec<String>>,
    
    #[validate(length(max = 20))]
    pub status: Option<String>,
    
    pub is_active: Option<bool>,
}

impl UpdateGuiaRequest {
    pub fn apply_to(self, mut guia: Guia, updated_by: Option<i32>) -> Guia {
        if let Some(id_persona) = self.id_persona {
            guia.id_persona = id_persona;
        }
        if let Some(nro_carnet) = self.nro_carnet {
            guia.nro_carnet = nro_carnet;
        }
        if let Some(idiomas) = self.idiomas {
            guia.idiomas = Some(serde_json::json!(idiomas));
        }
        if let Some(especialidades) = self.especialidades {
            guia.especialidades = Some(serde_json::json!(especialidades));
        }
        if let Some(status) = self.status {
            // Parse String to enum, keep old value if invalid
            if let Ok(status_enum) = status.parse::<StatusGuia>() {
                guia.status = status_enum;
            }
        }
        if let Some(is_active) = self.is_active {
            guia.is_active = is_active;
        }
        guia.updated_by = updated_by;
        guia.updated_at = Utc::now();
        guia
    }
}

/// DTO para listado de guías con información de la persona asociada
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct GuiaListItemDto {
    pub id: i32,
    pub id_persona: i32,
    pub nro_carnet: String,
    #[ts(type = "object | null")]
    pub idiomas: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub especialidades: Option<JsonValue>,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Datos de la persona asociada
    pub persona_nombre: Option<String>,
    pub persona_apellidos: Option<String>,
    pub persona_nro_documento: Option<String>,
    pub persona_telefono: Option<String>,
    pub persona_correo: Option<String>,
}
