use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Agencia;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AgenciaResponse {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Agencia> for AgenciaResponse {
    fn from(a: Agencia) -> Self {
        Self {
            id: a.id,
            nombre: a.nombre,
            ruc: a.ruc,
            telefono: a.telefono,
            correo: a.correo,
            direccion: a.direccion,
            paleta_colores: a.paleta_colores,
            media: a.media,
            encargado: a.encargado,
            is_active: a.is_active,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AgenciaListItemDto {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub encargado_nombre: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateAgenciaRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    #[validate(length(equal = 11, message = "RUC debe tener exactamente 11 dígitos"))]
    pub ruc: String,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    pub direccion: Option<String>,
    
    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
    
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    
    pub encargado: Option<i32>,
}

impl CreateAgenciaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Agencia {
        let now = Utc::now();
        Agencia {
            id: 0,
            nombre: self.nombre,
            ruc: self.ruc,
            telefono: self.telefono,
            correo: self.correo,
            direccion: self.direccion,
            paleta_colores: self.paleta_colores,
            media: self.media,
            encargado: self.encargado,
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
pub struct UpdateAgenciaRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    #[validate(length(equal = 11))]
    pub ruc: Option<String>,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    pub direccion: Option<String>,
    
    #[ts(type = "object | null | undefined")]
    pub paleta_colores: Option<JsonValue>,
    
    #[ts(type = "object | null | undefined")]
    pub media: Option<JsonValue>,
    
    pub encargado: Option<i32>,
    
    pub is_active: Option<bool>,
}

impl UpdateAgenciaRequest {
    pub fn apply_to(self, mut agencia: Agencia, updated_by: Option<i32>) -> Agencia {
        if let Some(nombre) = self.nombre {
            agencia.nombre = nombre;
        }
        if let Some(ruc) = self.ruc {
            agencia.ruc = ruc;
        }
        if let Some(telefono) = self.telefono {
            agencia.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            agencia.correo = Some(correo);
        }
        if let Some(direccion) = self.direccion {
            agencia.direccion = Some(direccion);
        }
        if let Some(paleta) = self.paleta_colores {
            agencia.paleta_colores = Some(paleta);
        }
        if let Some(media) = self.media {
            agencia.media = Some(media);
        }
        // encargado: siempre actualizar (permite asignar y quitar encargado)
        agencia.encargado = self.encargado;
        
        if let Some(is_active) = self.is_active {
            agencia.is_active = is_active;
        }
        agencia.updated_by = updated_by;
        agencia.updated_at = Utc::now();
        agencia
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AgenciaListResponse {
    pub items: Vec<AgenciaResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Request para actualizar solo la interfaz de la agencia (logo y paleta de colores)
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateAgenciaInterfazRequest {
    #[ts(type = "object | null | undefined")]
    pub paleta_colores: Option<JsonValue>,
    
    #[ts(type = "object | null | undefined")]
    pub media: Option<JsonValue>,
}

impl UpdateAgenciaInterfazRequest {
    pub fn apply_to(self, mut agencia: Agencia, updated_by: Option<i32>) -> Agencia {
        if let Some(paleta) = self.paleta_colores {
            agencia.paleta_colores = Some(paleta);
        }
        if let Some(media) = self.media {
            agencia.media = Some(media);
        }
        agencia.updated_by = updated_by;
        agencia.updated_at = Utc::now();
        agencia
    }
}
