
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusFile {
    Pendiente,
    Confirmado,
    EnCurso,
    Completado,
    Cancelado,
}

impl std::fmt::Display for StatusFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusFile::Pendiente => write!(f, "pendiente"),
            StatusFile::Confirmado => write!(f, "confirmado"),
            StatusFile::EnCurso => write!(f, "en_curso"),
            StatusFile::Completado => write!(f, "completado"),
            StatusFile::Cancelado => write!(f, "cancelado"),
        }
    }
}

impl std::str::FromStr for StatusFile {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pendiente" => Ok(StatusFile::Pendiente),
            "confirmado" => Ok(StatusFile::Confirmado),
            "en_curso" => Ok(StatusFile::EnCurso),
            "completado" => Ok(StatusFile::Completado),
            "cancelado" => Ok(StatusFile::Cancelado),
            _ => Err(format!("Status de file inválido: {s}")),
        }
    }
}

impl Default for StatusFile {
    fn default() -> Self {
        StatusFile::Pendiente
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: i32,
    pub id_tour: i32,
    pub id_agencia: i32,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    pub notas: Option<String>,
    pub status: String,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl File {
    pub fn new(id_tour: i32, id_agencia: i32, fecha_inicio: NaiveDate, fecha_fin: NaiveDate) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_tour,
            id_agencia,
            fecha_inicio,
            fecha_fin,
            lugar_recojo: None,
            hora_recojo: None,
            notas: None,
            status: StatusFile::Pendiente.to_string(),
            monto_total: BigDecimal::from(0),
            monto_pagado: BigDecimal::from(0),
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Obtiene el status como enum
    pub fn get_status(&self) -> StatusFile {
        self.status.parse().unwrap_or_default()
    }
    
    /// Saldo pendiente
    pub fn saldo_pendiente(&self) -> BigDecimal {
        &self.monto_total - &self.monto_pagado
    }
    
    /// Verifica si está pagado completamente
    pub fn esta_pagado(&self) -> bool {
        self.monto_pagado >= self.monto_total
    }
    
    /// Verifica si se puede cancelar
    pub fn puede_cancelar(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Pendiente | StatusFile::Confirmado)
    }
    
    /// Duración en días
    pub fn duracion_dias(&self) -> i64 {
        (self.fecha_fin - self.fecha_inicio).num_days() + 1
    }
}
