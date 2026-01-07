//! Service para que usuarios (guías, conductores, restaurantes) vean sus files asignados
//! Utiliza consultas SQL optimizadas con JOINs para evitar N+1 queries

use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use tracing::{info, instrument};

use crate::application::dtos::{MyFileAsGuiaDto, MyFileAsConductorDto, MyFileAsRestauranteDto};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;

/// Port para consultas de "mis files"
#[async_trait]
pub trait MyFilesRepositoryPort: Send + Sync {
    /// Obtiene files asignados a un guía (por id_persona del guía)
    async fn find_files_for_guia(&self, id_persona: i32) -> Result<Vec<MyFileAsGuiaDto>, ApplicationError>;
    
    /// Obtiene files asignados a un conductor (por id_persona del conductor)
    async fn find_files_for_conductor(&self, id_persona: i32) -> Result<Vec<MyFileAsConductorDto>, ApplicationError>;
    
    /// Obtiene files asignados a un restaurante (por id del restaurante)
    async fn find_files_for_restaurante(&self, id_restaurante: i32) -> Result<Vec<MyFileAsRestauranteDto>, ApplicationError>;
}

/// Implementación del repositorio usando raw SQL para JOINs eficientes
pub struct PostgresMyFilesRepository {
    pool: DatabasePool,
}

impl PostgresMyFilesRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MyFilesRepositoryPort for PostgresMyFilesRepository {
    #[instrument(skip(self))]
    async fn find_files_for_guia(&self, id_persona: i32) -> Result<Vec<MyFileAsGuiaDto>, ApplicationError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        // SQL con JOIN eficiente: guias -> file_guias -> files -> tours -> agencias
        let query = diesel::sql_query(r#"
            SELECT 
                f.id as file_id,
                f.file_code,
                f.fecha_inicio::text as fecha_inicio,
                f.fecha_fin::text as fecha_fin,
                f.lugar_recojo,
                f.hora_recojo::text as hora_recojo,
                f.status,
                f.nro_pasajeros,
                t.id as tour_id,
                t.nombre as tour_nombre,
                t.lugar_inicio as tour_lugar_inicio,
                t.lugar_fin as tour_lugar_fin,
                a.id as agencia_id,
                a.nombre as agencia_nombre,
                fg.rol as rol_guia,
                fg.created_at as asignado_at
            FROM guias g
            INNER JOIN file_guias fg ON fg.id_guia = g.id
            INNER JOIN files f ON f.id = fg.id_file
            INNER JOIN tours t ON t.id = f.id_tour
            INNER JOIN agencias a ON a.id = f.id_agencia
            WHERE g.id_persona = $1
              AND f.is_active = true
            ORDER BY f.fecha_inicio DESC, f.hora_recojo ASC
        "#)
        .bind::<Integer, _>(id_persona);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            file_id: i32,
            #[diesel(sql_type = Nullable<Text>)]
            file_code: Option<String>,
            #[diesel(sql_type = Text)]
            fecha_inicio: String,
            #[diesel(sql_type = Text)]
            fecha_fin: String,
            #[diesel(sql_type = Nullable<Text>)]
            lugar_recojo: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            hora_recojo: Option<String>,
            #[diesel(sql_type = Text)]
            status: String,
            #[diesel(sql_type = Integer)]
            nro_pasajeros: i32,
            #[diesel(sql_type = Integer)]
            tour_id: i32,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Text)]
            tour_lugar_inicio: String,
            #[diesel(sql_type = Text)]
            tour_lugar_fin: String,
            #[diesel(sql_type = Integer)]
            agencia_id: i32,
            #[diesel(sql_type = Text)]
            agencia_nombre: String,
            #[diesel(sql_type = Nullable<Text>)]
            rol_guia: Option<String>,
            #[diesel(sql_type = Timestamptz)]
            asignado_at: chrono::DateTime<Utc>,
        }
        
        let rows: Vec<RawRow> = query
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando files para guía: {}", e)))?;
        
        let results: Vec<MyFileAsGuiaDto> = rows.into_iter().map(|r| MyFileAsGuiaDto {
            file_id: r.file_id,
            file_code: r.file_code,
            fecha_inicio: r.fecha_inicio,
            fecha_fin: r.fecha_fin,
            lugar_recojo: r.lugar_recojo,
            hora_recojo: r.hora_recojo,
            status: r.status,
            nro_pasajeros: r.nro_pasajeros,
            tour_id: r.tour_id,
            tour_nombre: r.tour_nombre,
            tour_lugar_inicio: r.tour_lugar_inicio,
            tour_lugar_fin: r.tour_lugar_fin,
            agencia_id: r.agencia_id,
            agencia_nombre: r.agencia_nombre,
            rol_guia: r.rol_guia,
            asignado_at: r.asignado_at,
        }).collect();
        
        info!("📋 Encontrados {} files para guía (persona: {})", results.len(), id_persona);
        Ok(results)
    }
    
    #[instrument(skip(self))]
    async fn find_files_for_conductor(&self, id_persona: i32) -> Result<Vec<MyFileAsConductorDto>, ApplicationError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        // SQL con JOIN eficiente: conductores -> file_vehiculos -> files -> tours -> agencias, vehiculos
        let query = diesel::sql_query(r#"
            SELECT 
                f.id as file_id,
                f.file_code,
                f.fecha_inicio::text as fecha_inicio,
                f.fecha_fin::text as fecha_fin,
                f.lugar_recojo,
                f.hora_recojo::text as hora_recojo,
                f.status,
                f.nro_pasajeros,
                t.id as tour_id,
                t.nombre as tour_nombre,
                t.lugar_inicio as tour_lugar_inicio,
                t.lugar_fin as tour_lugar_fin,
                a.id as agencia_id,
                a.nombre as agencia_nombre,
                v.id as vehiculo_id,
                v.nombre as vehiculo_nombre,
                v.placa as vehiculo_placa,
                v.capacidad as vehiculo_capacidad,
                fv.created_at as asignado_at
            FROM conductores c
            INNER JOIN file_vehiculos fv ON fv.id_conductor = c.id
            INNER JOIN vehiculos v ON v.id = fv.id_vehiculo
            INNER JOIN files f ON f.id = fv.id_file
            INNER JOIN tours t ON t.id = f.id_tour
            INNER JOIN agencias a ON a.id = f.id_agencia
            WHERE c.id_persona = $1
              AND f.is_active = true
            ORDER BY f.fecha_inicio DESC, f.hora_recojo ASC
        "#)
        .bind::<Integer, _>(id_persona);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            file_id: i32,
            #[diesel(sql_type = Nullable<Text>)]
            file_code: Option<String>,
            #[diesel(sql_type = Text)]
            fecha_inicio: String,
            #[diesel(sql_type = Text)]
            fecha_fin: String,
            #[diesel(sql_type = Nullable<Text>)]
            lugar_recojo: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            hora_recojo: Option<String>,
            #[diesel(sql_type = Text)]
            status: String,
            #[diesel(sql_type = Integer)]
            nro_pasajeros: i32,
            #[diesel(sql_type = Integer)]
            tour_id: i32,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Text)]
            tour_lugar_inicio: String,
            #[diesel(sql_type = Text)]
            tour_lugar_fin: String,
            #[diesel(sql_type = Integer)]
            agencia_id: i32,
            #[diesel(sql_type = Text)]
            agencia_nombre: String,
            #[diesel(sql_type = Integer)]
            vehiculo_id: i32,
            #[diesel(sql_type = Text)]
            vehiculo_nombre: String,
            #[diesel(sql_type = Text)]
            vehiculo_placa: String,
            #[diesel(sql_type = Integer)]
            vehiculo_capacidad: i32,
            #[diesel(sql_type = Timestamptz)]
            asignado_at: chrono::DateTime<Utc>,
        }
        
        let rows: Vec<RawRow> = query
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando files para conductor: {}", e)))?;
        
        let results: Vec<MyFileAsConductorDto> = rows.into_iter().map(|r| MyFileAsConductorDto {
            file_id: r.file_id,
            file_code: r.file_code,
            fecha_inicio: r.fecha_inicio,
            fecha_fin: r.fecha_fin,
            lugar_recojo: r.lugar_recojo,
            hora_recojo: r.hora_recojo,
            status: r.status,
            nro_pasajeros: r.nro_pasajeros,
            tour_id: r.tour_id,
            tour_nombre: r.tour_nombre,
            tour_lugar_inicio: r.tour_lugar_inicio,
            tour_lugar_fin: r.tour_lugar_fin,
            agencia_id: r.agencia_id,
            agencia_nombre: r.agencia_nombre,
            vehiculo_id: r.vehiculo_id,
            vehiculo_nombre: r.vehiculo_nombre,
            vehiculo_placa: r.vehiculo_placa,
            vehiculo_capacidad: r.vehiculo_capacidad,
            asignado_at: r.asignado_at,
        }).collect();
        
        info!("📋 Encontrados {} files para conductor (persona: {})", results.len(), id_persona);
        Ok(results)
    }
    
    #[instrument(skip(self))]
    async fn find_files_for_restaurante(&self, id_restaurante: i32) -> Result<Vec<MyFileAsRestauranteDto>, ApplicationError> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        // SQL con JOIN eficiente: file_restaurantes -> files -> tours -> agencias
        let query = diesel::sql_query(r#"
            SELECT 
                f.id as file_id,
                f.file_code,
                f.fecha_inicio::text as fecha_inicio,
                f.fecha_fin::text as fecha_fin,
                f.status,
                f.nro_pasajeros,
                t.id as tour_id,
                t.nombre as tour_nombre,
                a.id as agencia_id,
                a.nombre as agencia_nombre,
                fr.tipo_servicio,
                fr.dia,
                fr.created_at as asignado_at
            FROM file_restaurantes fr
            INNER JOIN files f ON f.id = fr.id_file
            INNER JOIN tours t ON t.id = f.id_tour
            INNER JOIN agencias a ON a.id = f.id_agencia
            WHERE fr.id_restaurante = $1
              AND f.is_active = true
            ORDER BY f.fecha_inicio DESC, fr.dia ASC
        "#)
        .bind::<Integer, _>(id_restaurante);
        
        #[derive(QueryableByName)]
        struct RawRow {
            #[diesel(sql_type = Integer)]
            file_id: i32,
            #[diesel(sql_type = Nullable<Text>)]
            file_code: Option<String>,
            #[diesel(sql_type = Text)]
            fecha_inicio: String,
            #[diesel(sql_type = Text)]
            fecha_fin: String,
            #[diesel(sql_type = Text)]
            status: String,
            #[diesel(sql_type = Integer)]
            nro_pasajeros: i32,
            #[diesel(sql_type = Integer)]
            tour_id: i32,
            #[diesel(sql_type = Text)]
            tour_nombre: String,
            #[diesel(sql_type = Integer)]
            agencia_id: i32,
            #[diesel(sql_type = Text)]
            agencia_nombre: String,
            #[diesel(sql_type = Nullable<Text>)]
            tipo_servicio: Option<String>,
            #[diesel(sql_type = Nullable<Integer>)]
            dia: Option<i32>,
            #[diesel(sql_type = Timestamptz)]
            asignado_at: chrono::DateTime<Utc>,
        }
        
        let rows: Vec<RawRow> = query
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando files para restaurante: {}", e)))?;
        
        let results: Vec<MyFileAsRestauranteDto> = rows.into_iter().map(|r| MyFileAsRestauranteDto {
            file_id: r.file_id,
            file_code: r.file_code,
            fecha_inicio: r.fecha_inicio,
            fecha_fin: r.fecha_fin,
            status: r.status,
            nro_pasajeros: r.nro_pasajeros,
            tour_id: r.tour_id,
            tour_nombre: r.tour_nombre,
            agencia_id: r.agencia_id,
            agencia_nombre: r.agencia_nombre,
            tipo_servicio: r.tipo_servicio,
            dia: r.dia,
            asignado_at: r.asignado_at,
        }).collect();
        
        info!("📋 Encontrados {} files para restaurante: {}", results.len(), id_restaurante);
        Ok(results)
    }
}

/// Servicio de alto nivel para "mis files"
pub struct MyFilesService {
    repository: Arc<dyn MyFilesRepositoryPort>,
}

impl MyFilesService {
    pub fn new(repository: Arc<dyn MyFilesRepositoryPort>) -> Self {
        Self { repository }
    }
    
    /// Obtiene files para un guía usando su id_persona
    pub async fn get_my_files_as_guia(&self, id_persona: i32) -> Result<Vec<MyFileAsGuiaDto>, ApplicationError> {
        self.repository.find_files_for_guia(id_persona).await
    }
    
    /// Obtiene files para un conductor usando su id_persona
    pub async fn get_my_files_as_conductor(&self, id_persona: i32) -> Result<Vec<MyFileAsConductorDto>, ApplicationError> {
        self.repository.find_files_for_conductor(id_persona).await
    }
    
    /// Obtiene files para un restaurante usando su id_restaurante (id_entidad del user)
    pub async fn get_my_files_as_restaurante(&self, id_restaurante: i32) -> Result<Vec<MyFileAsRestauranteDto>, ApplicationError> {
        self.repository.find_files_for_restaurante(id_restaurante).await
    }
}
