use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::Entrada;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct EntradaResponse {
    pub id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Array de IDs de tours asociados. null = disponible para todos los tours.
    #[ts(type = "number[] | null")]
    pub tours_asociados: Option<Vec<i32>>,
}

impl From<Entrada> for EntradaResponse {
    fn from(e: Entrada) -> Self {
        // Convertir JsonValue a Vec<i32>
        let tours_asociados = e.tours_asociados.and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_i64().map(|n| n as i32))
                    .collect()
            })
        });
        
        Self {
            id: e.id,
            nombre: e.nombre,
            descripcion: e.descripcion,
            is_active: e.is_active,
            created_at: e.created_at,
            updated_at: e.updated_at,
            tours_asociados,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateEntradaRequest {
    #[validate(length(min = 2, max = 200, message = "Nombre debe tener entre 2 y 200 caracteres"))]
    pub nombre: String,
    
    pub descripcion: Option<String>,
    
    /// Array de IDs de tours asociados. null = disponible para todos los tours.
    #[ts(type = "number[] | null")]
    pub tours_asociados: Option<Vec<i32>>,
}

impl CreateEntradaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Entrada {
        let now = Utc::now();
        let tours_json = self.tours_asociados.map(|ids| {
            JsonValue::Array(ids.into_iter().map(|id| JsonValue::Number(id.into())).collect())
        });
        
        Entrada {
            id: 0,
            nombre: self.nombre,
            descripcion: self.descripcion,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
            tours_asociados: tours_json,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateEntradaRequest {
    #[validate(length(min = 2, max = 200))]
    pub nombre: Option<String>,
    
    pub descripcion: Option<String>,
    
    pub is_active: Option<bool>,
    
    /// Array de IDs de tours asociados. null = disponible para todos los tours.
    #[ts(type = "number[] | null")]
    pub tours_asociados: Option<Vec<i32>>,
}

impl UpdateEntradaRequest {
    pub fn apply_to(self, mut entrada: Entrada, updated_by: Option<i32>) -> Entrada {
        if let Some(nombre) = self.nombre {
            entrada.nombre = nombre;
        }
        if let Some(descripcion) = self.descripcion {
            entrada.descripcion = Some(descripcion);
        }
        if let Some(is_active) = self.is_active {
            entrada.is_active = is_active;
        }
        if let Some(tours) = self.tours_asociados {
            entrada.tours_asociados = Some(
                JsonValue::Array(tours.into_iter().map(|id| JsonValue::Number(id.into())).collect())
            );
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
