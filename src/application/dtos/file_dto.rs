use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::File;
use super::geo_dto::GeoLocation;

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
    
    // === Campos de recojo por tour ===
    /// Turno del tour: manana, tarde, noche
    pub turno_tour: Option<String>,
    /// Lugar de recojo para este tour específico
    pub lugar_recojo: Option<String>,
    /// Hora de recojo para este tour específico
    pub hora_recojo: Option<NaiveTime>,
    /// Geolocalización del punto de recojo
    pub geo_recojo: Option<GeoLocation>,
    
    /// Estado del file_tour: reservado, confirmado, en_progreso, completado, cancelado
    pub status: String,
    
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
    /// Turno del tour: manana, tarde, noche
    #[validate(length(max = 30))]
    pub turno_tour: Option<String>,
    /// Lugar de recojo para este tour específico
    #[validate(length(max = 200))]
    pub lugar_recojo: Option<String>,
    /// Hora de recojo para este tour específico
    pub hora_recojo: Option<NaiveTime>,
    /// Estado del file_tour: reservado, confirmado, en_progreso, completado, cancelado (default: reservado)
    #[validate(length(max = 30))]
    pub status: Option<String>,
    /// Geolocalización del punto de recojo
    pub geo_recojo: Option<GeoLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct FileResponse {
    pub id: i32,
    /// Tours asignados al file (puede ser múltiples)
    /// Cada tour contiene turno_tour, lugar_recojo, hora_recojo
    pub tours: Vec<FileTourDto>,
    /// ID del tour principal (primer tour por orden) - para compatibilidad
    #[ts(type = "number | null")]
    pub id_tour: Option<i32>,
    pub id_entidad: i32,
    pub entidad: Option<String>,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: NaiveDate,
    // Nota: lugar_recojo, hora_recojo, turno_tour ahora están en cada tour (FileTourDto)
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
    pub deadline_confirmacion: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_by_name: Option<String>,
    pub updated_by_name: Option<String>,
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
            id_entidad: f.id_entidad,
            entidad: f.entidad,
            fecha_inicio: f.fecha_inicio,
            fecha_fin: f.fecha_fin,
            // lugar_recojo, hora_recojo, turno_tour ahora están en FileTourDto
            notas: f.notas,
            status: f.status,
            monto_total: f.monto_total,
            monto_pagado: f.monto_pagado,
            saldo_pendiente: saldo,
            nro_pasajeros: f.nro_pasajeros,
            file_code: f.file_code,
            deadline_confirmacion: f.deadline_confirmacion,
            is_active: f.is_active,
            created_at: f.created_at,
            updated_at: f.updated_at,
            created_by: f.created_by,
            updated_by: f.updated_by,
            created_by_name: None,
            updated_by_name: None,
        }
    }

    /// Sets the user names for created_by and updated_by
    pub fn with_user_names(mut self, created_by_name: Option<String>, updated_by_name: Option<String>) -> Self {
        self.created_by_name = created_by_name;
        self.updated_by_name = updated_by_name;
        self
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
    /// Cada tour puede tener su propio turno_tour, lugar_recojo, hora_recojo
    /// Opcional si se usa id_tour (compatibilidad)
    pub tours: Option<Vec<FileTourInput>>,
    
    /// ID de tour único (para compatibilidad con código existente)
    /// Si se especifica, se ignora si también se especifica `tours`
    pub id_tour: Option<i32>,
    
    /// ID de la entidad (agencia u hotel) - Opcional si el usuario tiene rol vinculado (se auto-asigna)
    /// Requerido si el usuario es superadmin/admin
    pub id_entidad: Option<i32>,
    
    pub fecha_inicio: NaiveDate,
    
    pub fecha_fin: NaiveDate,
    
    // Nota: lugar_recojo, hora_recojo, turno_tour ahora están en FileTourInput (por tour)
    
    pub notas: Option<String>,
    
    #[validate(range(min = 0.0, message = "Monto debe ser positivo"))]
    pub monto_total: f64,
    
    #[validate(range(min = 0, message = "Número de pasajeros debe ser positivo"))]
    pub nro_pasajeros: Option<i32>,
    
    #[validate(length(max = 50))]
    pub file_code: Option<String>,
    
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
                turno_tour: None,
                lugar_recojo: None,
                hora_recojo: None,
                status: None,
                geo_recojo: None,
            }]
        } else {
            vec![]
        }
    }
    
    /// Convierte el request en una entidad File
    /// `id_entidad_resolved` es el ID de entidad ya resuelto (puede venir del request o del usuario)
    pub fn into_entity(self, created_by: Option<i32>, id_entidad_resolved: i32) -> File {
        let now = Utc::now();
        File {
            id: 0,
            id_entidad: id_entidad_resolved,
            entidad: None,
            fecha_inicio: self.fecha_inicio,
            fecha_fin: self.fecha_fin,
            // lugar_recojo, hora_recojo, turno_tour ahora están en file_tours
            notas: self.notas,
            status: "pendiente".to_string(),
            monto_total: BigDecimal::try_from(self.monto_total).unwrap_or_default(),
            monto_pagado: BigDecimal::from(0),
            nro_pasajeros: self.nro_pasajeros.unwrap_or(0),
            file_code: self.file_code,
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
    /// Cada tour puede tener su propio turno_tour, lugar_recojo, hora_recojo
    pub tours: Option<Vec<FileTourInput>>,
    
    /// ID de tour único (para compatibilidad) - se ignora si se especifica `tours`
    pub id_tour: Option<i32>,
    
    pub id_entidad: Option<i32>,
    
    pub fecha_inicio: Option<NaiveDate>,
    
    pub fecha_fin: Option<NaiveDate>,
    
    // Nota: lugar_recojo, hora_recojo, turno_tour ahora están en FileTourInput (por tour)
    
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
                turno_tour: None,
                lugar_recojo: None,
                hora_recojo: None,
                status: None,
                geo_recojo: None,
            }]);
        }
        None
    }
    
    pub fn apply_to(self, mut file: File, updated_by: Option<i32>) -> File {
        // Nota: tours se manejan aparte en el servicio
        if let Some(id_entidad) = self.id_entidad {
            file.id_entidad = id_entidad;
        }
        if let Some(fecha_inicio) = self.fecha_inicio {
            file.fecha_inicio = fecha_inicio;
        }
        if let Some(fecha_fin) = self.fecha_fin {
            file.fecha_fin = fecha_fin;
        }
        // Nota: lugar_recojo, hora_recojo, turno_tour ahora se manejan en file_tours
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

// =============================================================================
// CONFIRMACIÓN DE RESERVA
// =============================================================================

/// Request para confirmar una reserva (file)
/// 
/// Al confirmar una reserva:
/// - El status del file pasa a "confirmado"
/// - Se crea un pago_file pendiente para el contador de la agencia
/// - Se notifica a los admins
/// - Se registra en el log de actividad
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ConfirmReservaRequest {
    /// ID del file a confirmar
    pub file_id: i32,
    
    /// Monto total confirmado (puede ser diferente al estimado inicial)
    pub monto_total: Option<f64>,
    
    /// Días de plazo para el vencimiento del pago (default: 7 días)
    #[validate(range(min = 1, max = 90))]
    pub dias_vencimiento: Option<i32>,
    
    /// Notas adicionales para la confirmación
    #[validate(length(max = 500))]
    pub notas: Option<String>,
}

/// Response de confirmación de reserva
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ConfirmReservaResponse {
    /// File actualizado
    pub file: FileResponse,
    
    /// IDs de los pagos pendientes generados (uno por file_tour)
    pub pago_file_ids: Vec<i32>,
    
    /// Monto total a pagar
    #[ts(type = "string")]
    pub monto_total: BigDecimal,
    
    /// Fecha de vencimiento del pago
    pub fecha_vencimiento: String,
    
    /// Mensaje de confirmación
    pub mensaje: String,
}

/// Request para actualizar la hora de recojo de un file_tour
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileTourHoraRecojoRequest {
    /// Nueva hora de recojo (formato HH:MM:SS o HH:MM)
    pub hora_recojo: Option<NaiveTime>,
}

/// Response para actualización de hora de recojo
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileTourHoraRecojoResponse {
    pub success: bool,
    pub mensaje: String,
    /// La hora de recojo anterior (si existía)
    pub old_hora_recojo: Option<NaiveTime>,
    /// La nueva hora de recojo
    pub new_hora_recojo: Option<NaiveTime>,
}

/// Request para actualizar información de recojo de un file_tour (hora, lugar y/o geo)
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileTourRecojoRequest {
    /// Nueva hora de recojo (formato HH:MM:SS o HH:MM)
    pub hora_recojo: Option<NaiveTime>,
    /// Nuevo lugar de recojo
    #[validate(length(max = 200))]
    pub lugar_recojo: Option<String>,
    /// Nueva geolocalización de recojo
    pub geo_recojo: Option<GeoLocation>,
}

/// Response para actualización de información de recojo
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateFileTourRecojoResponse {
    pub success: bool,
    pub mensaje: String,
    /// La hora de recojo anterior (si existía)
    pub old_hora_recojo: Option<NaiveTime>,
    /// La nueva hora de recojo
    pub new_hora_recojo: Option<NaiveTime>,
    /// El lugar de recojo anterior (si existía)
    pub old_lugar_recojo: Option<String>,
    /// El nuevo lugar de recojo
    pub new_lugar_recojo: Option<String>,
    /// La geolocalización de recojo anterior (si existía)
    pub old_geo_recojo: Option<GeoLocation>,
    /// La nueva geolocalización de recojo
    pub new_geo_recojo: Option<GeoLocation>,
}

