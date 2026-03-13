use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Hotel;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct HotelResponse {
    pub id: i32,
    pub id_cadena: i32,
    pub nombre: String,
    pub categoria: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub ciudad: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Hotel> for HotelResponse {
    fn from(h: Hotel) -> Self {
        Self {
            id: h.id,
            id_cadena: h.id_cadena,
            nombre: h.nombre,
            categoria: h.categoria,
            telefono: h.telefono,
            correo: h.correo,
            direccion: h.direccion,
            ciudad: h.ciudad,
            is_active: h.is_active,
            created_at: h.created_at,
            updated_at: h.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct HotelListItemDto {
    pub id: i32,
    pub id_cadena: i32,
    pub cadena_nombre: Option<String>,
    pub nombre: String,
    pub categoria: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub ciudad: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateHotelRequest {
    pub id_cadena: i32,

    #[validate(length(
        min = 2,
        max = 200,
        message = "Nombre debe tener entre 2 y 200 caracteres"
    ))]
    pub nombre: String,

    #[validate(length(max = 50))]
    pub categoria: Option<String>,

    #[validate(length(max = 20))]
    pub telefono: Option<String>,

    #[validate(email)]
    pub correo: Option<String>,

    pub direccion: Option<String>,

    #[validate(length(max = 100))]
    pub ciudad: Option<String>,
}

impl CreateHotelRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Hotel {
        let now = Utc::now();
        Hotel {
            id: 0,
            id_cadena: self.id_cadena,
            nombre: self.nombre,
            categoria: self.categoria,
            telefono: self.telefono,
            correo: self.correo,
            direccion: self.direccion,
            ciudad: self.ciudad,
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
pub struct UpdateHotelRequest {
    pub id_cadena: Option<i32>,

    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,

    #[validate(length(max = 50))]
    pub categoria: Option<String>,

    #[validate(length(max = 20))]
    pub telefono: Option<String>,

    #[validate(email)]
    pub correo: Option<String>,

    pub direccion: Option<String>,

    #[validate(length(max = 100))]
    pub ciudad: Option<String>,

    pub is_active: Option<bool>,
}

impl UpdateHotelRequest {
    pub fn apply_to(self, mut hotel: Hotel, updated_by: Option<i32>) -> Hotel {
        if let Some(id_cadena) = self.id_cadena {
            hotel.id_cadena = id_cadena;
        }
        if let Some(nombre) = self.nombre {
            hotel.nombre = nombre;
        }
        if let Some(categoria) = self.categoria {
            hotel.categoria = Some(categoria);
        }
        if let Some(telefono) = self.telefono {
            hotel.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            hotel.correo = Some(correo);
        }
        if let Some(direccion) = self.direccion {
            hotel.direccion = Some(direccion);
        }
        if let Some(ciudad) = self.ciudad {
            hotel.ciudad = Some(ciudad);
        }
        if let Some(is_active) = self.is_active {
            hotel.is_active = is_active;
        }
        hotel.updated_by = updated_by;
        hotel.updated_at = Utc::now();
        hotel
    }
}
