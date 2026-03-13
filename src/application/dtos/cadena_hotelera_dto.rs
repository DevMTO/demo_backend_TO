use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::CadenaHotelera;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CadenaHoteleraResponse {
    pub id: i32,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CadenaHotelera> for CadenaHoteleraResponse {
    fn from(c: CadenaHotelera) -> Self {
        Self {
            id: c.id,
            nombre: c.nombre,
            telefono: c.telefono,
            correo: c.correo,
            media: c.media,
            encargado: c.encargado,
            is_active: c.is_active,
            paleta_colores: c.paleta_colores,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CadenaHoteleraListItemDto {
    pub id: i32,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,
    pub encargado: Option<i32>,
    pub encargado_nombre: Option<String>,
    pub is_active: bool,
    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
    pub total_hoteles: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateCadenaHoteleraRequest {
    #[validate(length(
        min = 2,
        max = 200,
        message = "Nombre debe tener entre 2 y 200 caracteres"
    ))]
    pub nombre: String,

    #[validate(length(max = 20))]
    pub telefono: Option<String>,

    #[validate(email)]
    pub correo: Option<String>,

    #[ts(type = "object | null")]
    pub media: Option<JsonValue>,

    pub encargado: Option<i32>,

    #[ts(type = "object | null")]
    pub paleta_colores: Option<JsonValue>,
}

impl CreateCadenaHoteleraRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> CadenaHotelera {
        let now = Utc::now();
        CadenaHotelera {
            id: 0,
            nombre: self.nombre,
            telefono: self.telefono,
            correo: self.correo,
            media: self.media,
            encargado: self.encargado,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
            paleta_colores: self.paleta_colores,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateCadenaHoteleraRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,

    #[validate(length(max = 20))]
    pub telefono: Option<String>,

    #[validate(email)]
    pub correo: Option<String>,

    #[ts(type = "object | null | undefined")]
    pub media: Option<JsonValue>,

    pub encargado: Option<i32>,

    pub is_active: Option<bool>,

    #[ts(type = "object | null | undefined")]
    pub paleta_colores: Option<JsonValue>,
}

impl UpdateCadenaHoteleraRequest {
    pub fn apply_to(self, mut cadena: CadenaHotelera, updated_by: Option<i32>) -> CadenaHotelera {
        if let Some(nombre) = self.nombre {
            cadena.nombre = nombre;
        }
        if let Some(telefono) = self.telefono {
            cadena.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            cadena.correo = Some(correo);
        }
        if let Some(media) = self.media {
            cadena.media = Some(media);
        }
        cadena.encargado = self.encargado;
        if let Some(is_active) = self.is_active {
            cadena.is_active = is_active;
        }
        if let Some(paleta_colores) = self.paleta_colores {
            cadena.paleta_colores = Some(paleta_colores);
        }
        cadena.updated_by = updated_by;
        cadena.updated_at = Utc::now();
        cadena
    }
}

/// Request para actualizar solo la interfaz de la cadena hotelera (logo y paleta de colores)
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateCadenaInterfazRequest {
    #[ts(type = "object | null | undefined")]
    pub paleta_colores: Option<JsonValue>,
    
    #[ts(type = "object | null | undefined")]
    pub media: Option<JsonValue>,
}

impl UpdateCadenaInterfazRequest {
    pub fn apply_to(self, mut cadena: CadenaHotelera, updated_by: Option<i32>) -> CadenaHotelera {
        if let Some(paleta) = self.paleta_colores {
            cadena.paleta_colores = Some(paleta);
        }
        if let Some(media) = self.media {
            cadena.media = Some(media);
        }
        cadena.updated_by = updated_by;
        cadena.updated_at = Utc::now();
        cadena
    }
}
