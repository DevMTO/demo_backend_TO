use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use validator::Validate;

use crate::domain::entities::File;

#[derive(Debug, Clone, Serialize)]
pub struct FileResponse {
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
    pub saldo_pendiente: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<File> for FileResponse {
    fn from(f: File) -> Self {
        let saldo = f.monto_total.clone() - f.monto_pagado.clone();
        Self {
            id: f.id,
            id_tour: f.id_tour,
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
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateFileRequest {
    pub id_tour: i32,
    
    pub id_agencia: i32,
    
    pub fecha_inicio: NaiveDate,
    
    pub fecha_fin: NaiveDate,
    
    #[validate(length(max = 200))]
    pub lugar_recojo: Option<String>,
    
    pub hora_recojo: Option<NaiveTime>,
    
    pub notas: Option<String>,
    
    #[validate(range(min = 0.0, message = "Monto debe ser positivo"))]
    pub monto_total: f64,
}

impl CreateFileRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> File {
        let now = Utc::now();
        File {
            id: 0,
            id_tour: self.id_tour,
            id_agencia: self.id_agencia,
            fecha_inicio: self.fecha_inicio,
            fecha_fin: self.fecha_fin,
            lugar_recojo: self.lugar_recojo,
            hora_recojo: self.hora_recojo,
            notas: self.notas,
            status: "pendiente".to_string(),
            monto_total: BigDecimal::try_from(self.monto_total).unwrap_or_default(),
            monto_pagado: BigDecimal::from(0),
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateFileRequest {
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
}

impl UpdateFileRequest {
    pub fn apply_to(self, mut file: File, updated_by: Option<i32>) -> File {
        if let Some(id_tour) = self.id_tour {
            file.id_tour = id_tour;
        }
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
        file.updated_by = updated_by;
        file.updated_at = Utc::now();
        file
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileListResponse {
    pub items: Vec<FileResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
