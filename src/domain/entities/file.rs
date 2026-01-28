
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// Estados del flujo de trabajo de un File:
/// - Pendiente: Estado inicial (esperando confirmación)
/// - Reservado: File con reserva confirmada
/// - Asignado: File con recursos asignados (guías, vehículos)
/// - Confirmado: File completamente confirmado y listo
/// - EnCurso: El tour está en progreso
/// - Completado: El tour finalizó exitosamente
/// - Cancelado: File cancelado por el cliente o agencia
/// - Anulado: File anulado por incumplimiento o fuerza mayor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusFile {
    Pendiente,
    Reservado,
    Asignado,
    Confirmado,
    EnCurso,
    Completado,
    Cancelado,
    Anulado,
}

impl std::fmt::Display for StatusFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusFile::Pendiente => write!(f, "pendiente"),
            StatusFile::Reservado => write!(f, "reservado"),
            StatusFile::Asignado => write!(f, "asignado"),
            StatusFile::Confirmado => write!(f, "confirmado"),
            StatusFile::EnCurso => write!(f, "en_curso"),
            StatusFile::Completado => write!(f, "completado"),
            StatusFile::Cancelado => write!(f, "cancelado"),
            StatusFile::Anulado => write!(f, "anulado"),
        }
    }
}

impl std::str::FromStr for StatusFile {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pendiente" => Ok(StatusFile::Pendiente),
            "reservado" => Ok(StatusFile::Reservado),
            "asignado" => Ok(StatusFile::Asignado),
            "confirmado" => Ok(StatusFile::Confirmado),
            "en_curso" => Ok(StatusFile::EnCurso),
            "completado" => Ok(StatusFile::Completado),
            "cancelado" => Ok(StatusFile::Cancelado),
            "anulado" => Ok(StatusFile::Anulado),
            _ => Err(format!("Status de file inválido: {}. Valores: pendiente, reservado, asignado, confirmado, en_curso, completado, cancelado, anulado", s)),
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
    // id_tour eliminado - ahora los tours están en file_tours (relación N:M)
    pub id_agencia: i32,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub notas: Option<String>,
    pub status: String,
    pub monto_total: BigDecimal,
    pub monto_pagado: BigDecimal,
    pub nro_pasajeros: i32,
    pub file_code: Option<String>,
    pub deadline_confirmacion: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

impl File {
    pub fn new(id_agencia: i32, fecha_inicio: NaiveDate, fecha_fin: NaiveDate) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            // tours se asignan aparte en file_tours
            id_agencia,
            fecha_inicio,
            fecha_fin,
            notas: None,
            status: StatusFile::Pendiente.to_string(),
            monto_total: BigDecimal::from(0),
            monto_pagado: BigDecimal::from(0),
            nro_pasajeros: 0,
            file_code: None, // Será asignado después de insertar
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
        matches!(status, StatusFile::Pendiente | StatusFile::Reservado | StatusFile::Confirmado | StatusFile::Asignado)
    }
    
    /// Verifica si se puede cancelar
    pub fn puede_cancelar(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Pendiente | StatusFile::Reservado | StatusFile::Confirmado | StatusFile::Asignado)
    }
    
    /// Verifica si se puede confirmar
    pub fn puede_confirmar(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Pendiente | StatusFile::Reservado)
    }
    
    /// Verifica si se puede asignar recursos
    pub fn puede_asignar(&self) -> bool {
        let status = self.get_status();
        matches!(status, StatusFile::Pendiente | StatusFile::Reservado | StatusFile::Confirmado)
    }
    
    /// Duración en días
    pub fn duracion_dias(&self) -> i64 {
        (self.fecha_fin - self.fecha_inicio).num_days() + 1
    }
}
