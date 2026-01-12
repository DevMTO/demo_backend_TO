
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// Estados del flujo de trabajo de un File:
/// - Reservado: Estado inicial al crear el file
/// - Confirmado: File confirmado antes del deadline
/// - Asignado: File con recursos asignados (guías, vehículos)
/// - EnCurso: El tour está en progreso
/// - Completado: El tour finalizó exitosamente
/// - Anulado: No se confirmó a tiempo o fue cancelado
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusFile {
    Reservado,
    Confirmado,
    Asignado,
    EnCurso,
    Completado,
    Anulado,
}

impl std::fmt::Display for StatusFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusFile::Reservado => write!(f, "reservado"),
            StatusFile::Confirmado => write!(f, "confirmado"),
            StatusFile::Asignado => write!(f, "asignado"),
            StatusFile::EnCurso => write!(f, "en_curso"),
            StatusFile::Completado => write!(f, "completado"),
            StatusFile::Anulado => write!(f, "anulado"),
        }
    }
}

impl std::str::FromStr for StatusFile {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reservado" => Ok(StatusFile::Reservado),
            "confirmado" => Ok(StatusFile::Confirmado),
            "asignado" => Ok(StatusFile::Asignado),
            "en_curso" => Ok(StatusFile::EnCurso),
            "completado" => Ok(StatusFile::Completado),
            "anulado" => Ok(StatusFile::Anulado),
            // Backward compatibility
            "pendiente" => Ok(StatusFile::Reservado),
            "cancelado" => Ok(StatusFile::Anulado),
            _ => Err(format!("Status de file inválido: {s}")),
        }
    }
}

impl Default for StatusFile {
    fn default() -> Self {
        StatusFile::Reservado
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
    pub nro_pasajeros: i32,
    pub file_code: Option<String>,
    pub turno_tour: Option<String>,
    pub deadline_confirmacion: Option<DateTime<Utc>>,
    pub is_active: bool,
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
            status: StatusFile::Reservado.to_string(),
            monto_total: BigDecimal::from(0),
            monto_pagado: BigDecimal::from(0),
            nro_pasajeros: 0,
            file_code: None, // Será asignado después de insertar
            turno_tour: None,
            deadline_confirmacion: None,
            is_active: true,
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
    
    /// Verifica si se puede anular
    pub fn puede_anular(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Reservado | StatusFile::Confirmado | StatusFile::Asignado)
    }
    
    /// Verifica si se puede confirmar
    pub fn puede_confirmar(&self) -> bool {
        self.get_status() == StatusFile::Reservado
    }
    
    /// Verifica si se puede asignar recursos
    pub fn puede_asignar(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Reservado | StatusFile::Confirmado)
    }
    
    /// Duración en días
    pub fn duracion_dias(&self) -> i64 {
        (self.fecha_fin - self.fecha_inicio).num_days() + 1
    }
}
