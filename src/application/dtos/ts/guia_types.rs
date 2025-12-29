use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum StatusGuiaTs {
    Disponible,
    EnServicio,
    Descanso,
    Inactivo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaTs {
    pub id: Uuid,
    pub codigo: String,
    pub id_persona: Uuid,
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub status: StatusGuiaTs,
    pub id_agencia: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateGuiaRequestTs {
    pub id_persona: Uuid,
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub id_agencia: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuiaRequestTs {
    pub numero_carnet: Option<String>,
    pub idiomas: Option<Vec<String>>,
    pub especialidades: Option<Vec<String>>,
    pub fecha_vencimiento_carnet: Option<NaiveDate>,
    pub status: Option<StatusGuiaTs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaListResponseTs {
    pub guias: Vec<GuiaTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct GuiaDetailTs {
    pub guia: GuiaTs,
    pub persona_nombre_completo: String,
    pub persona_documento: Option<String>,
    pub persona_telefono: Option<String>,
    pub persona_email: Option<String>,
}
