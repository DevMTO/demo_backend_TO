use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

// ============== Enums ==============

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum StatusVehiculoTs {
    Disponible,
    EnUso,
    Mantenimiento,
    Inactivo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "lowercase")]
pub enum StatusConductorTs {
    Disponible,
    EnServicio,
    Descanso,
    Inactivo,
}

// ============== Transporte ==============

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct TransporteTs {
    pub id: Uuid,
    pub codigo: String,
    pub descripcion: Option<String>,
    pub id_vehiculo: Option<Uuid>,
    pub id_conductor: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateTransporteRequestTs {
    pub descripcion: Option<String>,
    pub id_vehiculo: Option<Uuid>,
    pub id_conductor: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransporteRequestTs {
    pub descripcion: Option<String>,
    pub id_vehiculo: Option<Uuid>,
    pub id_conductor: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct TransporteListResponseTs {
    pub transportes: Vec<TransporteTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ============== Vehículo ==============

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct VehiculoTs {
    pub id: Uuid,
    pub placa: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub anio: Option<i32>,
    pub color: Option<String>,
    pub capacidad: Option<i32>,
    pub tipo_vehiculo: Option<String>,
    pub status: StatusVehiculoTs,
    pub id_agencia: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateVehiculoRequestTs {
    pub placa: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub anio: Option<i32>,
    pub color: Option<String>,
    pub capacidad: Option<i32>,
    pub tipo_vehiculo: Option<String>,
    pub id_agencia: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateVehiculoRequestTs {
    pub placa: Option<String>,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub anio: Option<i32>,
    pub color: Option<String>,
    pub capacidad: Option<i32>,
    pub tipo_vehiculo: Option<String>,
    pub status: Option<StatusVehiculoTs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct VehiculoListResponseTs {
    pub vehiculos: Vec<VehiculoTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ============== Conductor ==============

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct ConductorTs {
    pub id: Uuid,
    pub codigo: String,
    pub id_persona: Uuid,
    pub numero_licencia: Option<String>,
    pub categoria_licencia: Option<String>,
    pub fecha_vencimiento_licencia: Option<NaiveDate>,
    pub status: StatusConductorTs,
    pub id_agencia: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateConductorRequestTs {
    pub id_persona: Uuid,
    pub numero_licencia: Option<String>,
    pub categoria_licencia: Option<String>,
    pub fecha_vencimiento_licencia: Option<NaiveDate>,
    pub id_agencia: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateConductorRequestTs {
    pub numero_licencia: Option<String>,
    pub categoria_licencia: Option<String>,
    pub fecha_vencimiento_licencia: Option<NaiveDate>,
    pub status: Option<StatusConductorTs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct ConductorListResponseTs {
    pub conductores: Vec<ConductorTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct ConductorDetailTs {
    pub conductor: ConductorTs,
    pub persona_nombre_completo: String,
    pub persona_documento: Option<String>,
    pub persona_telefono: Option<String>,
}
