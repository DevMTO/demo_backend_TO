use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Transporte;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TransporteResponse {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub encargado: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Transporte> for TransporteResponse {
    fn from(t: Transporte) -> Self {
        Self {
            id: t.id,
            nombre: t.nombre,
            ruc: t.ruc,
            telefono: t.telefono,
            correo: t.correo,
            direccion: t.direccion,
            encargado: t.encargado,
            is_active: t.is_active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// DTO para listar transportes con el nombre del encargado
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TransporteListItemDto {
    pub id: i32,
    pub nombre: String,
    pub ruc: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub encargado: Option<i32>,
    pub encargado_nombre: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateTransporteRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    #[validate(length(equal = 11, message = "RUC debe tener exactamente 11 dígitos"))]
    pub ruc: String,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    pub direccion: Option<String>,
    
    pub encargado: Option<i32>,
}

impl CreateTransporteRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Transporte {
        let now = Utc::now();
        Transporte {
            id: 0,
            nombre: self.nombre,
            ruc: self.ruc,
            telefono: self.telefono,
            correo: self.correo,
            direccion: self.direccion,
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
pub struct UpdateTransporteRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    #[validate(length(equal = 11))]
    pub ruc: Option<String>,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    #[validate(email)]
    pub correo: Option<String>,
    
    pub direccion: Option<String>,
    
    pub encargado: Option<i32>,
    
    pub is_active: Option<bool>,
}

impl UpdateTransporteRequest {
    pub fn apply_to(self, mut transporte: Transporte, updated_by: Option<i32>) -> Transporte {
        if let Some(nombre) = self.nombre {
            transporte.nombre = nombre;
        }
        if let Some(ruc) = self.ruc {
            transporte.ruc = ruc;
        }
        if let Some(telefono) = self.telefono {
            transporte.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            transporte.correo = Some(correo);
        }
        if let Some(direccion) = self.direccion {
            transporte.direccion = Some(direccion);
        }
        if let Some(encargado) = self.encargado {
            transporte.encargado = Some(encargado);
        }
        if let Some(is_active) = self.is_active {
            transporte.is_active = is_active;
        }
        transporte.updated_by = updated_by;
        transporte.updated_at = Utc::now();
        transporte
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct TransporteListResponse {
    pub items: Vec<TransporteResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
