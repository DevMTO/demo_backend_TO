//! Servicio para consultar "Mis Pagos" de proveedores
//!
//! Este servicio permite que guías, conductores y restaurantes vean
//! sus pagos pendientes y pagados.

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Numeric, Text, Timestamptz};
use diesel_async::RunQueryDsl;
use tracing::instrument;

use crate::application::dtos::contabilidad_dto::{
    MiPagoGuiaResponse, MiPagoConductorResponse, MiPagoRestauranteResponse,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;

// ============================================================================
// REPOSITORY TRAIT
// ============================================================================

#[async_trait]
pub trait MisPagosRepositoryPort: Send + Sync {
    /// Obtiene los pagos para un guía (basado en id_persona)
    async fn find_pagos_guia(&self, id_persona: i32) -> Result<Vec<MiPagoGuiaResponse>, ApplicationError>;
    
    /// Obtiene los pagos para un conductor (basado en id_persona)
    async fn find_pagos_conductor(&self, id_persona: i32) -> Result<Vec<MiPagoConductorResponse>, ApplicationError>;
    
    /// Obtiene los pagos para un restaurante (basado en id_restaurante)
    async fn find_pagos_restaurante(&self, id_restaurante: i32) -> Result<Vec<MiPagoRestauranteResponse>, ApplicationError>;
}

// ============================================================================
// REPOSITORY IMPLEMENTATION
// ============================================================================

pub struct PostgresMisPagosRepository {
    pool: DatabasePool,
}

impl PostgresMisPagosRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MisPagosRepositoryPort for PostgresMisPagosRepository {
    #[instrument(skip(self))]
    async fn find_pagos_guia(&self, id_persona: i32) -> Result<Vec<MiPagoGuiaResponse>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Query: Obtener pagos donde el guía (por id_persona) está asignado
        // Relación: pagos_proveedores.id_file_guia -> file_guias.id -> file_guias.id_file_tour -> file_tours.id
        let query = diesel::sql_query(r#"
            SELECT 
                pp.id as id_pago,
                COALESCE(fg.id, 0) as id_file_guia,
                COALESCE(f.file_code, 'N/A') as file_code,
                COALESCE(t.nombre, 'Sin tour asignado') as tour_nombre,
                COALESCE(ft.fecha_tour::text, '') as fecha_tour,
                pp.monto_total as monto,
                pp.estado,
                pp.fecha_pago,
                pp.comprobante_url
            FROM guias g
            INNER JOIN pagos_proveedores pp ON pp.id_guia = g.id AND pp.tipo_proveedor = 'guia'
            LEFT JOIN file_guias fg ON fg.id = pp.id_file_guia
            LEFT JOIN file_tours ft ON ft.id = fg.id_file_tour
            LEFT JOIN files f ON f.id = ft.id_file
            LEFT JOIN tours t ON t.id = ft.id_tour
            WHERE g.id_persona = $1
            ORDER BY pp.created_at DESC
        "#)
        .bind::<Integer, _>(id_persona);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            id_pago: i32,
            #[diesel(sql_type = Integer)]
            id_file_guia: i32,
            #[diesel(sql_type = Text)]
            file_code: String,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Text)]
            fecha_tour: String,
            #[diesel(sql_type = Numeric)]
            monto: BigDecimal,
            #[diesel(sql_type = Text)]
            estado: String,
            #[diesel(sql_type = Nullable<Timestamptz>)]
            fecha_pago: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = Nullable<Text>)]
            comprobante_url: Option<String>,
        }
        
        let rows = query
            .load::<RawRow>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando pagos de guía: {}", e)))?;
        
        let result: Vec<MiPagoGuiaResponse> = rows
            .into_iter()
            .map(|row| MiPagoGuiaResponse {
                id_pago: row.id_pago,
                id_file_guia: row.id_file_guia,
                file_code: if row.file_code == "N/A" { None } else { Some(row.file_code) },
                tour_nombre: row.tour_nombre,
                fecha_tour: if row.fecha_tour.is_empty() { None } else { Some(row.fecha_tour) },
                monto: row.monto,
                estado: row.estado,
                fecha_pago: row.fecha_pago,
                comprobante_url: row.comprobante_url,
            })
            .collect();
        
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn find_pagos_conductor(&self, id_persona: i32) -> Result<Vec<MiPagoConductorResponse>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Query: Obtener pagos donde el conductor está asociado al transporte que recibe el pago
        // Relación: pagos_proveedores.id_file_vehiculo -> file_vehiculos.id -> file_vehiculos.id_file_tour -> file_tours.id
        let query = diesel::sql_query(r#"
            SELECT 
                pp.id as id_pago,
                COALESCE(fv.id, 0) as id_file_vehiculo,
                COALESCE(f.file_code, 'N/A') as file_code,
                COALESCE(t.nombre, 'Sin tour asignado') as tour_nombre,
                COALESCE(v.placa, 'N/A') as vehiculo_placa,
                COALESCE(ft.fecha_tour::text, '') as fecha_tour,
                pp.monto_total as monto,
                pp.estado,
                pp.fecha_pago,
                pp.comprobante_url
            FROM conductores c
            INNER JOIN transportes tr ON tr.id = c.id_transporte
            INNER JOIN pagos_proveedores pp ON pp.id_transporte = tr.id AND pp.tipo_proveedor = 'transporte'
            LEFT JOIN file_vehiculos fv ON fv.id = pp.id_file_vehiculo
            LEFT JOIN vehiculos v ON v.id = fv.id_vehiculo
            LEFT JOIN file_tours ft ON ft.id = fv.id_file_tour
            LEFT JOIN files f ON f.id = ft.id_file
            LEFT JOIN tours t ON t.id = ft.id_tour
            WHERE c.id_persona = $1
            ORDER BY pp.created_at DESC
        "#)
        .bind::<Integer, _>(id_persona);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            id_pago: i32,
            #[diesel(sql_type = Integer)]
            id_file_vehiculo: i32,
            #[diesel(sql_type = Text)]
            file_code: String,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Text)]
            vehiculo_placa: String,
            #[diesel(sql_type = Text)]
            fecha_tour: String,
            #[diesel(sql_type = Numeric)]
            monto: BigDecimal,
            #[diesel(sql_type = Text)]
            estado: String,
            #[diesel(sql_type = Nullable<Timestamptz>)]
            fecha_pago: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = Nullable<Text>)]
            comprobante_url: Option<String>,
        }
        
        let rows = query
            .load::<RawRow>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando pagos de conductor: {}", e)))?;
        
        let result: Vec<MiPagoConductorResponse> = rows
            .into_iter()
            .map(|row| MiPagoConductorResponse {
                id_pago: row.id_pago,
                id_file_vehiculo: row.id_file_vehiculo,
                file_code: if row.file_code == "N/A" { None } else { Some(row.file_code) },
                tour_nombre: row.tour_nombre,
                vehiculo_placa: row.vehiculo_placa,
                fecha_tour: if row.fecha_tour.is_empty() { None } else { Some(row.fecha_tour) },
                monto: row.monto,
                estado: row.estado,
                fecha_pago: row.fecha_pago,
                comprobante_url: row.comprobante_url,
            })
            .collect();
        
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn find_pagos_restaurante(&self, id_restaurante: i32) -> Result<Vec<MiPagoRestauranteResponse>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Query: Obtener pagos para un restaurante específico
        // Relación: pagos_proveedores.id_file_restaurante -> file_restaurantes.id -> file_restaurantes.id_file_tour -> file_tours.id
        let query = diesel::sql_query(r#"
            SELECT 
                pp.id as id_pago,
                COALESCE(fr.id, 0) as id_file_restaurante,
                COALESCE(f.file_code, 'N/A') as file_code,
                COALESCE(t.nombre, 'Sin tour asignado') as tour_nombre,
                COALESCE(ft.fecha_tour::text, '') as fecha_tour,
                fr.tipo_servicio,
                pp.monto_total as monto,
                pp.estado,
                pp.fecha_pago,
                pp.comprobante_url
            FROM pagos_proveedores pp
            LEFT JOIN file_restaurantes fr ON fr.id = pp.id_file_restaurante
            LEFT JOIN file_tours ft ON ft.id = fr.id_file_tour
            LEFT JOIN files f ON f.id = ft.id_file
            LEFT JOIN tours t ON t.id = ft.id_tour
            WHERE pp.id_restaurante = $1 
              AND pp.tipo_proveedor = 'restaurante'
            ORDER BY pp.created_at DESC
        "#)
        .bind::<Integer, _>(id_restaurante);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            id_pago: i32,
            #[diesel(sql_type = Integer)]
            id_file_restaurante: i32,
            #[diesel(sql_type = Text)]
            file_code: String,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Text)]
            fecha_tour: String,
            #[diesel(sql_type = Nullable<Text>)]
            tipo_servicio: Option<String>,
            #[diesel(sql_type = Numeric)]
            monto: BigDecimal,
            #[diesel(sql_type = Text)]
            estado: String,
            #[diesel(sql_type = Nullable<Timestamptz>)]
            fecha_pago: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = Nullable<Text>)]
            comprobante_url: Option<String>,
        }
        
        let rows = query
            .load::<RawRow>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando pagos de restaurante: {}", e)))?;
        
        let result: Vec<MiPagoRestauranteResponse> = rows
            .into_iter()
            .map(|row| MiPagoRestauranteResponse {
                id_pago: row.id_pago,
                id_file_restaurante: row.id_file_restaurante,
                file_code: if row.file_code == "N/A" { None } else { Some(row.file_code) },
                tour_nombre: row.tour_nombre,
                fecha_tour: if row.fecha_tour.is_empty() { None } else { Some(row.fecha_tour) },
                tipo_servicio: row.tipo_servicio,
                monto: row.monto,
                estado: row.estado,
                fecha_pago: row.fecha_pago,
                comprobante_url: row.comprobante_url,
            })
            .collect();
        
        Ok(result)
    }
}

// ============================================================================
// SERVICE
// ============================================================================

use std::sync::Arc;

pub struct MisPagosService {
    repository: Arc<dyn MisPagosRepositoryPort>,
}

impl MisPagosService {
    pub fn new(repository: Arc<dyn MisPagosRepositoryPort>) -> Self {
        Self { repository }
    }
    
    /// Obtiene los pagos para un guía (basado en id_persona del usuario)
    #[instrument(skip(self))]
    pub async fn get_mis_pagos_guia(&self, id_persona: i32) -> Result<Vec<MiPagoGuiaResponse>, ApplicationError> {
        self.repository.find_pagos_guia(id_persona).await
    }
    
    /// Obtiene los pagos para un conductor (basado en id_persona del usuario)
    #[instrument(skip(self))]
    pub async fn get_mis_pagos_conductor(&self, id_persona: i32) -> Result<Vec<MiPagoConductorResponse>, ApplicationError> {
        self.repository.find_pagos_conductor(id_persona).await
    }
    
    /// Obtiene los pagos para un restaurante (basado en id_entidad del usuario que es el id_restaurante)
    #[instrument(skip(self))]
    pub async fn get_mis_pagos_restaurante(&self, id_restaurante: i32) -> Result<Vec<MiPagoRestauranteResponse>, ApplicationError> {
        self.repository.find_pagos_restaurante(id_restaurante).await
    }
}
