use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Entrada;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct EntradaResponse {
    pub id: i32,
    pub nombre: String,
    pub ruta: Option<String>,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Entrada> for EntradaResponse {
    fn from(e: Entrada) -> Self {
        Self {
            id: e.id,
            nombre: e.nombre,
            ruta: e.ruta,
            descripcion: e.descripcion,
            is_active: e.is_active,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateEntradaRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    #[validate(length(max = 200))]
    pub ruta: Option<String>,
    
    pub descripcion: Option<String>,
}

impl CreateEntradaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Entrada {
        let now = Utc::now();
        Entrada {
            id: 0,
            nombre: self.nombre,
            ruta: self.ruta,
            descripcion: self.descripcion,
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
pub struct UpdateEntradaRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    #[validate(length(max = 200))]
    pub ruta: Option<String>,
    
    pub descripcion: Option<String>,
    
    pub is_active: Option<bool>,
}

impl UpdateEntradaRequest {
    pub fn apply_to(self, mut entrada: Entrada, updated_by: Option<i32>) -> Entrada {
        if let Some(nombre) = self.nombre {
            entrada.nombre = nombre;
        }
        if let Some(ruta) = self.ruta {
            entrada.ruta = Some(ruta);
        }
        if let Some(descripcion) = self.descripcion {
            entrada.descripcion = Some(descripcion);
        }
        if let Some(is_active) = self.is_active {
            entrada.is_active = is_active;
        }
        entrada.updated_by = updated_by;
        entrada.updated_at = Utc::now();
        entrada
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntradaListResponse {
    pub items: Vec<EntradaResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
