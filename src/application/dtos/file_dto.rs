use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::File;

/// DTO para un tour dentro de un file (relación N:M) con información del tour
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileTourDto {
    /// ID de la relación file_tour
    pub id: i32,
    /// ID del tour
    pub id_tour: i32,
    /// Orden del tour dentro del file
    pub orden: i32,
    /// Precio aplicado (puede ser diferente al precio base del tour)
    #[ts(type = "string | null")]
    pub precio_aplicado: Option<BigDecimal>,
    /// Notas específicas para este tour en este file
    pub notas: Option<String>,
    /// Fecha específica del tour (puede ser diferente para cada tour del file)
    pub fecha_tour: Option<NaiveDate>,
    
    // === Información del tour (INNER JOIN) ===
    /// Nombre del tour
    pub tour_nombre: Option<String>,
    /// Lugar de inicio del tour
    pub tour_lugar_inicio: Option<String>,
    /// Lugar de fin del tour
    pub tour_lugar_fin: Option<String>,
    /// Precio base del tour (referencia)
    #[ts(type = "string | null")]
    pub tour_precio_base: Option<BigDecimal>,
    /// Duración en días del tour
    pub tour_duracion_dias: Option<i32>,
    /// Tipo de tour (full day, half day, etc)
    pub tour_tipo: Option<String>,
    /// Si el tour está activo
    pub tour_is_active: Option<bool>,
}

/// DTO para crear/asignar un tour a un file
#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileTourInput {
    pub id_tour: i32,
    /// Orden del tour (1, 2, 3...). Si no se especifica, se asigna automáticamente.
    pub orden: Option<i32>,
    /// Precio específico para este tour en este file. Si no se especifica, se usa precio_base del tour.
    pub precio_aplicado: Option<f64>,
    pub notas: Option<String>,
    /// Fecha específica del tour (si es diferente a la fecha del file)
    pub fecha_tour: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileResponse {
    pub id: i32,
    /// Tours asignados al file (puede ser múltiples)
    pub tours: Vec<FileTourDto>,
    /// ID del tour principal (primer tour por orden) - para compatibilidad
    #[ts(type = "number | null")]
    pub id_tour: Option<i32>,
    pub id_agencia: i32,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    pub notas: Option<String>,
    pub status: String,
    #[ts(type = "string")]
    pub monto_total: BigDecimal,
    #[ts(type = "string")]
    pub monto_pagado: BigDecimal,
    #[ts(type = "string")]
    pub saldo_pendiente: BigDecimal,
    pub nro_pasajeros: i32,
    pub file_code: Option<String>,
    pub turno_tour: Option<String>,
    pub deadline_confirmacion: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FileResponse {
    /// Crea un FileResponse a partir de un File y sus tours asociados
    pub fn from_file_with_tours(f: File, tours: Vec<FileTourDto>) -> Self {
        let saldo = f.monto_total.clone() - f.monto_pagado.clone();
        // El tour principal es el primero por orden
        let id_tour = tours.iter().min_by_key(|t| t.orden).map(|t| t.id_tour);
        Self {
            id: f.id,
            tours,
            id_tour,
            id_agencia: f.id_agencia,
            fecha_inicio: f.fecha_inicio,
            fecha_fin: f.fecha_fin,
            lugar_recojo: f.lugar_recojo,
            hora_recojo: f.hora_recojo,
            notas: f.notas,
            status: f.status,
            monto_total: f.monto_total,
            monto_pagado: f.monto_pagado,
            saldo_pendiente: saldo,
            nro_pasajeros: f.nro_pasajeros,
            file_code: f.file_code,
            turno_tour: f.turno_tour,
            deadline_confirmacion: f.deadline_confirmacion,
            is_active: f.is_active,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

impl From<File> for FileResponse {
    fn from(f: File) -> Self {
        // Cuando no tenemos tours cargados, devolvemos lista vacía
        Self::from_file_with_tours(f, vec![])
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateFileRequest {
    /// Tours a asignar al file (puede ser uno o múltiples)
    /// Opcional si se usa id_tour (compatibilidad)
    pub tours: Option<Vec<FileTourInput>>,
    
    /// ID de tour único (para compatibilidad con código existente)
    /// Si se especifica, se ignora si también se especifica `tours`
    pub id_tour: Option<i32>,
    
    /// ID de la agencia - Opcional si el usuario es rol "agencias" (se auto-asigna)
    /// Requerido si el usuario es superadmin/admin
    pub id_agencia: Option<i32>,
    
    pub fecha_inicio: NaiveDate,
    
    pub fecha_fin: NaiveDate,
    
    #[validate(length(max = 200))]
    pub lugar_recojo: Option<String>,
    
    pub hora_recojo: Option<NaiveTime>,
    
    pub notas: Option<String>,
    
    #[validate(range(min = 0.0, message = "Monto debe ser positivo"))]
    pub monto_total: f64,
    
    #[validate(range(min = 0, message = "Número de pasajeros debe ser positivo"))]
    pub nro_pasajeros: Option<i32>,
    
    #[validate(length(max = 50))]
    pub file_code: Option<String>,
    
    #[validate(length(max = 30))]
    pub turno_tour: Option<String>,
    
    /// Fecha límite para confirmar el file (opcional)
    pub deadline_confirmacion: Option<DateTime<Utc>>,
}

impl CreateFileRequest {
    /// Obtiene los tours a crear. Si `tours` está vacío pero `id_tour` tiene valor,
    /// crea un tour único para compatibilidad.
    pub fn get_tours(&self) -> Vec<FileTourInput> {
        // Primero verificar si hay tours especificados
        if let Some(ref tours) = self.tours {
            if !tours.is_empty() {
                return tours.clone();
            }
        }
        // Compatibilidad: si no hay tours pero hay id_tour, crear uno
        if let Some(id_tour) = self.id_tour {
            vec![FileTourInput {
                id_tour,
                orden: Some(1),
                precio_aplicado: None,
                notas: None,
                fecha_tour: None,
            }]
        } else {
            vec![]
        }
    }
    
    /// Convierte el request en una entidad File
    /// `id_agencia_resolved` es el ID de agencia ya resuelto (puede venir del request o del usuario)
    pub fn into_entity(self, created_by: Option<i32>, id_agencia_resolved: i32) -> File {
        let now = Utc::now();
        File {
            id: 0,
            id_agencia: id_agencia_resolved,
            fecha_inicio: self.fecha_inicio,
            fecha_fin: self.fecha_fin,
            lugar_recojo: self.lugar_recojo,
            hora_recojo: self.hora_recojo,
            notas: self.notas,
            status: "reservado".to_string(),
            monto_total: BigDecimal::try_from(self.monto_total).unwrap_or_default(),
            monto_pagado: BigDecimal::from(0),
            nro_pasajeros: self.nro_pasajeros.unwrap_or(0),
            file_code: self.file_code,
            turno_tour: self.turno_tour,
            deadline_confirmacion: self.deadline_confirmacion,
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
pub struct UpdateFileRequest {
    /// Tours a actualizar/reemplazar (opcional)
    /// Si se especifica, reemplaza todos los tours existentes
    pub tours: Option<Vec<FileTourInput>>,
    
    /// ID de tour único (para compatibilidad) - se ignora si se especifica `tours`
    pub id_tour: Option<i32>,
    
    pub id_agencia: Option<i32>,
    
    pub fecha_inicio: Option<NaiveDate>,
    
    pub fecha_fin: Option<NaiveDate>,
    
    #[validate(length(max = 200))]
    pub lugar_recojo: Option<String>,
    
    pub hora_recojo: Option<NaiveTime>,
    
    pub notas: Option<String>,
    
    #[validate(length(max = 30))]
    pub status: Option<String>,
    
    #[validate(range(min = 0.0))]
    pub monto_total: Option<f64>,
    
    #[validate(range(min = 0.0))]
    pub monto_pagado: Option<f64>,
    
    #[validate(range(min = 0))]
    pub nro_pasajeros: Option<i32>,
    
    #[validate(length(max = 50))]
    pub file_code: Option<String>,
    
    #[validate(length(max = 30))]
    pub turno_tour: Option<String>,
    
    pub deadline_confirmacion: Option<DateTime<Utc>>,
    
    pub is_active: Option<bool>,
}

impl UpdateFileRequest {
    /// Obtiene los tours a actualizar si se especificaron
    pub fn get_tours(&self) -> Option<Vec<FileTourInput>> {
        if let Some(ref tours) = self.tours {
            if !tours.is_empty() {
                return Some(tours.clone());
            }
        }
        // Compatibilidad: si no hay tours pero hay id_tour, crear uno
        if let Some(id_tour) = self.id_tour {
            return Some(vec![FileTourInput {
                id_tour,
                orden: Some(1),
                precio_aplicado: None,
                notas: None,
                fecha_tour: None,
            }]);
        }
        None
    }
    
    pub fn apply_to(self, mut file: File, updated_by: Option<i32>) -> File {
        // Nota: tours se manejan aparte en el servicio
        if let Some(id_agencia) = self.id_agencia {
            file.id_agencia = id_agencia;
        }
        if let Some(fecha_inicio) = self.fecha_inicio {
            file.fecha_inicio = fecha_inicio;
        }
        if let Some(fecha_fin) = self.fecha_fin {
            file.fecha_fin = fecha_fin;
        }
        if let Some(lugar_recojo) = self.lugar_recojo {
            file.lugar_recojo = Some(lugar_recojo);
        }
        if let Some(hora_recojo) = self.hora_recojo {
            file.hora_recojo = Some(hora_recojo);
        }
        if let Some(notas) = self.notas {
            file.notas = Some(notas);
        }
        if let Some(status) = self.status {
            file.status = status;
        }
        if let Some(monto_total) = self.monto_total {
            file.monto_total = BigDecimal::try_from(monto_total).unwrap_or_default();
        }
        if let Some(monto_pagado) = self.monto_pagado {
            file.monto_pagado = BigDecimal::try_from(monto_pagado).unwrap_or_default();
        }
        if let Some(nro_pasajeros) = self.nro_pasajeros {
            file.nro_pasajeros = nro_pasajeros;
        }
        if let Some(file_code) = self.file_code {
            file.file_code = Some(file_code);
        }
        if let Some(turno_tour) = self.turno_tour {
            file.turno_tour = Some(turno_tour);
        }
        if let Some(deadline_confirmacion) = self.deadline_confirmacion {
            file.deadline_confirmacion = Some(deadline_confirmacion);
        }
        if let Some(is_active) = self.is_active {
            file.is_active = is_active;
        }
        file.updated_by = updated_by;
        file.updated_at = Utc::now();
        file
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileListResponse {
    pub items: Vec<FileResponse>,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub page: i64,
    #[ts(type = "number")]
    pub page_size: i64,
    #[ts(type = "number")]
    pub total_pages: i64,
}
