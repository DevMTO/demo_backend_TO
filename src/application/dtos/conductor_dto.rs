use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::{Conductor, StatusConductor};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ConductorResponse {
    pub id: i32,
    pub id_persona: i32,
    pub id_transporte: Option<i32>,
    pub nro_brevete: String,
    pub tiene_soat: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Conductor> for ConductorResponse {
    fn from(c: Conductor) -> Self {
        Self {
            id: c.id,
            id_persona: c.id_persona,
            id_transporte: c.id_transporte,
            nro_brevete: c.nro_brevete,
            tiene_soat: c.tiene_soat,
            status: c.status.to_string(), // Enum → String
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateConductorRequest {
    pub id_persona: i32,
    
    pub id_transporte: Option<i32>,
    
    #[validate(length(min = 6, max = 20, message = "Número de brevete inválido"))]
    pub nro_brevete: String,
    
    pub tiene_soat: bool,
}

impl CreateConductorRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Conductor {
        let now = Utc::now();
        Conductor {
            id: 0,
            id_persona: self.id_persona,
            id_transporte: self.id_transporte,
            nro_brevete: self.nro_brevete,
            tiene_soat: self.tiene_soat,
            status: StatusConductor::Disponible,
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
pub struct UpdateConductorRequest {
    pub id_persona: Option<i32>,
    
    pub id_transporte: Option<i32>,
    
    #[validate(length(min = 6, max = 20))]
    pub nro_brevete: Option<String>,
    
    pub tiene_soat: Option<bool>,
    
    #[validate(length(max = 20))]
    pub status: Option<String>,
}

impl UpdateConductorRequest {
    pub fn apply_to(self, mut conductor: Conductor, updated_by: Option<i32>) -> Conductor {
        if let Some(id_persona) = self.id_persona {
            conductor.id_persona = id_persona;
        }
        if let Some(id_transporte) = self.id_transporte {
            conductor.id_transporte = Some(id_transporte);
        }
        if let Some(nro_brevete) = self.nro_brevete {
            conductor.nro_brevete = nro_brevete;
        }
        if let Some(tiene_soat) = self.tiene_soat {
            conductor.tiene_soat = tiene_soat;
        }
        if let Some(status) = self.status {
            // Parse String to enum, keep old value if invalid
            if let Ok(status_enum) = status.parse::<StatusConductor>() {
                conductor.status = status_enum;
            }
        }
        conductor.updated_by = updated_by;
        conductor.updated_at = Utc::now();
        conductor
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConductorListResponse {
    pub items: Vec<ConductorResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
