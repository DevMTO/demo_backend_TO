use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
use diesel_async::RunQueryDsl;
use bigdecimal::BigDecimal;
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{
        FileEntradaModel, NewFileEntradaModel,
        FileGuiaModel, FileGuiaWithPersonaModel, NewFileGuiaModel,
        FilePasajeroModel, FilePasajeroWithPersonaModel, NewFilePasajeroModel,
        FileRestauranteModel, NewFileRestauranteModel,
        FileVehiculoModel, FileVehiculoWithPersonaModel, NewFileVehiculoModel,
        FileTourModel, NewFileTourModel, FileTourWithTourModel,
    },
    schema::{file_entradas, file_guias, file_pasajeros, file_restaurantes, file_vehiculos, file_tours, tours},
};

// Importar traits y structs de input desde application/ports
use crate::application::ports::{
    FileEntradaRepositoryPort, FileGuiaRepositoryPort, FilePasajeroRepositoryPort,
    FileRestauranteRepositoryPort, FileVehiculoRepositoryPort, FileTourRepositoryPort,
    FileTourInputData,
};

// ==================== MODELOS EXTENDIDOS ====================

/// Modelo para file_vehiculo con datos extendidos de file, tour, agencia, vehiculo y conductor
#[derive(Debug, Clone, QueryableByName)]
pub struct FileVehiculoWithDetailsModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file_tour: i32,
    #[diesel(sql_type = Integer)]
    pub id_vehiculo: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub id_conductor: Option<i32>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Integer)]
    pub capacidad_asignada: i32,
    /// Estado de la asignación: reservado, confirmado, cancelado
    #[diesel(sql_type = Text)]
    pub status: String,
    // Datos del file
    #[diesel(sql_type = Nullable<Text>)]
    pub file_code: Option<String>,
    #[diesel(sql_type = Text)]
    pub file_fecha_inicio: String,
    #[diesel(sql_type = Text)]
    pub file_fecha_fin: String,
    #[diesel(sql_type = Text)]
    pub file_status: String,
    #[diesel(sql_type = Integer)]
    pub file_nro_pasajeros: i32,
    // Datos del tour
    #[diesel(sql_type = Integer)]
    pub tour_id: i32,
    #[diesel(sql_type = Text)]
    pub tour_nombre: String,
    // Datos de la agencia
    #[diesel(sql_type = Integer)]
    pub agencia_id: i32,
    #[diesel(sql_type = Text)]
    pub agencia_nombre: String,
    // Datos del vehículo
    #[diesel(sql_type = Nullable<Text>)]
    pub vehiculo_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub vehiculo_placa: Option<String>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub vehiculo_capacidad: Option<i32>,
    // Datos del conductor
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_nombre: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub conductor_brevete: Option<String>,
}

// ==================== IMPLEMENTACIONES ====================

pub struct PostgresFileEntradaRepository {
    pool: DatabasePool,
}

impl PostgresFileEntradaRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileEntradaRepositoryPort for PostgresFileEntradaRepository {
    #[instrument(skip(self))]
    async fn add(&self, id_file_tour: i32, id_entrada: i32, cantidad: i32, id_entrada_precio: Option<i32>, created_by: Option<i32>) -> Result<FileEntradaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileEntradaModel {
            id_file_tour,
            id_entrada,
            cantidad,
            created_by,
            id_entrada_precio,
            status: Some("asignado"), // Auto-asignado al crear
        };
        
        let result = diesel::insert_into(file_entradas::table)
            .values(&new_record)
            .returning(FileEntradaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Entrada asignada a file_tour: file_tour={}, entrada={}, cantidad={}", id_file_tour, id_entrada, cantidad);
        Ok(result)
    }
    
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_entradas::table.filter(file_entradas::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileEntradaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_entradas::table
            .filter(file_entradas::id_file_tour.eq(id_file_tour))
            .select(FileEntradaModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FileEntradaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_entradas::table
            .filter(file_entradas::id.eq(id))
            .select(FileEntradaModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FileEntradaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_entradas::table.filter(file_entradas::id.eq(id)))
            .set(file_entradas::status.eq(status))
            .returning(FileEntradaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_entrada: {}", e)))?;
        
        info!("Status de file_entrada {} actualizado a '{}'", id, status);
        Ok(result)
    }
}

// ==================== FILE GUIA REPOSITORY ====================

pub struct PostgresFileGuiaRepository {
    pool: DatabasePool,
}

impl PostgresFileGuiaRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileGuiaRepositoryPort for PostgresFileGuiaRepository {
    #[instrument(skip(self))]
    async fn add(&self, id_file_tour: i32, id_guia: i32, rol: Option<&str>, created_by: Option<i32>) -> Result<FileGuiaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileGuiaModel {
            id_file_tour,
            id_guia,
            rol,
            created_by,
            estado_confirmacion: Some("aceptado"), // Auto-aceptado al asignar (igual que conductor)
            status: Some("reservado"), // Auto-reservado
        };
        
        let result = diesel::insert_into(file_guias::table)
            .values(&new_record)
            .returning(FileGuiaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Guía asignado a file_tour: file_tour={}, guia={}", id_file_tour, id_guia);
        Ok(result)
    }
    
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_guias::table.filter(file_guias::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FileGuiaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_guias::table
            .filter(file_guias::id.eq(id))
            .select(FileGuiaModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn is_guia_assigned(&self, id_guia: i32, id_file_tour: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = file_guias::table
            .filter(file_guias::id_guia.eq(id_guia))
            .filter(file_guias::id_file_tour.eq(id_file_tour))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FileGuiaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_guias::table.filter(file_guias::id.eq(id)))
            .set(file_guias::status.eq(status))
            .returning(FileGuiaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_guia: {}", e)))?;
        
        info!("Status de file_guia {} actualizado a '{}'", id, status);
        Ok(result)
    }
    
    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: crate::infrastructure::persistence::models::file_guia_model::UpdateFileGuiaModel) -> Result<FileGuiaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_guias::table.filter(file_guias::id.eq(id)))
            .set(&data)
            .returning(FileGuiaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando file_guia: {}", e)))?;
        
        info!("FileGuia {} actualizado", id);
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn find_by_file_tour_with_persona(&self, id_file_tour: i32) -> Result<Vec<FileGuiaWithPersonaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // JOIN: file_guias -> guias -> personas
        let query = diesel::sql_query(r#"
            SELECT 
                fg.id,
                fg.id_file_tour,
                fg.id_guia,
                fg.rol,
                fg.created_at,
                fg.created_by,
                fg.estado_confirmacion,
                fg.confirmado_at,
                fg.motivo_rechazo,
                fg.status,
                g.nro_carnet as guia_nro_carnet,
                g.idiomas as guia_idiomas,
                p.nombre as guia_nombre,
                p.apellidos as guia_apellidos,
                p.telefono as guia_telefono,
                p.correo as guia_correo
            FROM file_guias fg
            INNER JOIN guias g ON g.id = fg.id_guia
            LEFT JOIN personas p ON p.id = g.id_persona
            WHERE fg.id_file_tour = $1
            ORDER BY fg.created_at ASC
        "#)
        .bind::<Integer, _>(id_file_tour);
        
        query.load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error obteniendo guías con persona: {}", e)))
    }
}

// ==================== FILE PASAJERO REPOSITORY ====================

pub struct PostgresFilePasajeroRepository {
    pool: DatabasePool,
}

impl PostgresFilePasajeroRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FilePasajeroRepositoryPort for PostgresFilePasajeroRepository {
    #[instrument(skip(self))]
    async fn add(&self, id_file: i32, id_persona: Option<i32>, asiento: Option<&str>, tipo_pasajero: Option<&str>, nacionalidad: Option<&str>, notas: Option<&str>, edad: Option<i32>, created_by: Option<i32>) -> Result<FilePasajeroModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFilePasajeroModel {
            id_file,
            id_persona,
            asiento,
            tipo_pasajero,
            notas,
            created_by,
            nacionalidad,
            edad,
            status: None, // Usa el default de la DB: 'reservado'
        };
        
        let result = diesel::insert_into(file_pasajeros::table)
            .values(&new_record)
            .returning(FilePasajeroModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let persona_info = id_persona.map(|p| format!("persona={}", p)).unwrap_or_else(|| "anónimo".to_string());
        info!("Pasajero agregado a file: file={}, {}", id_file, persona_info);
        Ok(result)
    }
    
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_pasajeros::table.filter(file_pasajeros::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn find_by_file_with_persona(&self, id_file: i32) -> Result<Vec<FilePasajeroWithPersonaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Usando LEFT JOIN para incluir pasajeros anónimos (sin id_persona)
        let query = diesel::sql_query(r#"
            SELECT 
                fp.id,
                fp.id_file,
                fp.id_persona,
                fp.asiento,
                fp.tipo_pasajero,
                fp.notas,
                fp.created_at,
                fp.created_by,
                fp.nacionalidad,
                fp.edad,
                fp.status,
                p.nombre as pasajero_nombre,
                p.apellidos as pasajero_apellidos,
                p.tipo_documento as pasajero_tipo_documento,
                p.nro_documento as pasajero_documento,
                p.telefono as pasajero_telefono
            FROM file_pasajeros fp
            LEFT JOIN personas p ON p.id = fp.id_persona
            WHERE fp.id_file = $1
            ORDER BY fp.created_at ASC
        "#)
        .bind::<Integer, _>(id_file);
        
        query.load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error obteniendo pasajeros con persona: {}", e)))
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FilePasajeroModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_pasajeros::table
            .filter(file_pasajeros::id.eq(id))
            .select(FilePasajeroModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn count_by_file(&self, id_file: i32) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_pasajeros::table
            .filter(file_pasajeros::id_file.eq(id_file))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FilePasajeroModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_pasajeros::table.filter(file_pasajeros::id.eq(id)))
            .set(file_pasajeros::status.eq(status))
            .returning(FilePasajeroModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_pasajero: {}", e)))?;
        
        info!("Status de file_pasajero {} actualizado a '{}'", id, status);
        Ok(result)
    }
    
    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: crate::infrastructure::persistence::models::file_pasajero_model::UpdateFilePasajeroModel) -> Result<FilePasajeroModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_pasajeros::table.filter(file_pasajeros::id.eq(id)))
            .set(&data)
            .returning(FilePasajeroModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando file_pasajero: {}", e)))?;
        
        info!("FilePasajero {} actualizado", id);
        Ok(result)
    }
}

// ==================== FILE RESTAURANTE REPOSITORY ====================

pub struct PostgresFileRestauranteRepository {
    pool: DatabasePool,
}

impl PostgresFileRestauranteRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRestauranteRepositoryPort for PostgresFileRestauranteRepository {
    #[instrument(skip(self))]
    async fn add(&self, id_file_tour: i32, id_restaurante: i32, tipo_servicio: Option<&str>, precio: Option<BigDecimal>, created_by: Option<i32>) -> Result<FileRestauranteModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileRestauranteModel {
            id_file_tour,
            id_restaurante,
            tipo_servicio,
            created_by,
            precio,
            status: Some("asignado"), // Auto-asignado al crear
        };
        
        let result = diesel::insert_into(file_restaurantes::table)
            .values(&new_record)
            .returning(FileRestauranteModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Restaurante asignado a file_tour: file_tour={}, restaurante={}", id_file_tour, id_restaurante);
        Ok(result)
    }
    
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_restaurantes::table.filter(file_restaurantes::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileRestauranteModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_restaurantes::table
            .filter(file_restaurantes::id_file_tour.eq(id_file_tour))
            .select(FileRestauranteModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FileRestauranteModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_restaurantes::table
            .filter(file_restaurantes::id.eq(id))
            .select(FileRestauranteModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FileRestauranteModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_restaurantes::table.filter(file_restaurantes::id.eq(id)))
            .set(file_restaurantes::status.eq(status))
            .returning(FileRestauranteModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_restaurante: {}", e)))?;
        
        info!("Status de file_restaurante {} actualizado a '{}'", id, status);
        Ok(result)
    }
}

// ==================== FILE VEHICULO REPOSITORY ====================

pub struct PostgresFileVehiculoRepository {
    pool: DatabasePool,
}

impl PostgresFileVehiculoRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileVehiculoRepositoryPort for PostgresFileVehiculoRepository {
    #[instrument(skip(self))]
    async fn add(&self, id_file_tour: i32, id_vehiculo: i32, id_conductor: Option<i32>, capacidad_asignada: i32, created_by: Option<i32>) -> Result<FileVehiculoModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileVehiculoModel {
            id_file_tour,
            id_vehiculo,
            id_conductor,
            capacidad_asignada,
            created_by,
            status: None, // Usa el default de la DB: 'reservado'
        };
        
        let result = diesel::insert_into(file_vehiculos::table)
            .values(&new_record)
            .returning(FileVehiculoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Vehículo asignado a file_tour: file_tour={}, vehiculo={}, conductor={:?}, capacidad={}", id_file_tour, id_vehiculo, id_conductor, capacidad_asignada);
        Ok(result)
    }
    
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_vehiculos::table.filter(file_vehiculos::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileVehiculoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_vehiculos::table
            .filter(file_vehiculos::id_file_tour.eq(id_file_tour))
            .select(FileVehiculoModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn find_all_with_details(&self) -> Result<Vec<FileVehiculoWithDetailsModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let query = diesel::sql_query(r#"
            SELECT 
                fv.id,
                fv.id_file_tour,
                fv.id_vehiculo,
                fv.id_conductor,
                fv.created_at,
                fv.capacidad_asignada,
                fv.status,
                f.file_code,
                f.fecha_inicio::text as file_fecha_inicio,
                f.fecha_fin::text as file_fecha_fin,
                f.status as file_status,
                f.nro_pasajeros as file_nro_pasajeros,
                t.id as tour_id,
                t.nombre as tour_nombre,
                a.id as agencia_id,
                a.nombre as agencia_nombre,
                v.nombre as vehiculo_nombre,
                v.placa as vehiculo_placa,
                v.capacidad as vehiculo_capacidad,
                CASE WHEN c.id IS NOT NULL THEN CONCAT(pc.nombre, ' ', pc.apellidos) ELSE NULL END as conductor_nombre,
                c.nro_brevete as conductor_brevete
            FROM file_vehiculos fv
            INNER JOIN file_tours ft ON ft.id = fv.id_file_tour
            INNER JOIN files f ON f.id = ft.id_file
            INNER JOIN tours t ON t.id = ft.id_tour
            INNER JOIN agencias a ON a.id = f.id_agencia
            INNER JOIN vehiculos v ON v.id = fv.id_vehiculo
            LEFT JOIN conductores c ON c.id = fv.id_conductor
            LEFT JOIN personas pc ON pc.id = c.id_persona
            WHERE f.is_active = true
            ORDER BY f.fecha_inicio DESC, fv.created_at DESC
        "#);
        
        let results = query
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error consultando file_vehiculos: {}", e)))?;
        
        info!("Encontrados {} file_vehiculos con detalles", results.len());
        Ok(results)
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FileVehiculoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_vehiculos::table
            .filter(file_vehiculos::id.eq(id))
            .select(FileVehiculoModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn find_files_by_vehiculo(&self, id_vehiculo: i32) -> Result<Vec<i32>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results: Vec<i32> = file_vehiculos::table
            .filter(file_vehiculos::id_vehiculo.eq(id_vehiculo))
            .select(file_vehiculos::id_file_tour)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results)
    }
    
    async fn is_vehiculo_assigned(&self, id_vehiculo: i32, id_file_tour: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = file_vehiculos::table
            .filter(file_vehiculos::id_vehiculo.eq(id_vehiculo))
            .filter(file_vehiculos::id_file_tour.eq(id_file_tour))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FileVehiculoModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_vehiculos::table.filter(file_vehiculos::id.eq(id)))
            .set(file_vehiculos::status.eq(status))
            .returning(FileVehiculoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_vehiculo: {}", e)))?;
        
        info!("Status de file_vehiculo {} actualizado a '{}'", id, status);
        Ok(result)
    }
    
    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: crate::infrastructure::persistence::models::file_vehiculo_model::UpdateFileVehiculoModel) -> Result<FileVehiculoModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_vehiculos::table.filter(file_vehiculos::id.eq(id)))
            .set(&data)
            .returning(FileVehiculoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando file_vehiculo: {}", e)))?;
        
        info!("FileVehiculo {} actualizado", id);
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn find_by_file_tour_with_persona(&self, id_file_tour: i32) -> Result<Vec<FileVehiculoWithPersonaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // JOIN: file_vehiculos -> vehiculos -> transportes
        //       file_vehiculos -> conductores -> personas
        let query = diesel::sql_query(r#"
            SELECT 
                fv.id,
                fv.id_file_tour,
                fv.id_vehiculo,
                fv.id_conductor,
                fv.capacidad_asignada,
                fv.created_at,
                fv.created_by,
                fv.status,
                v.nombre as vehiculo_nombre,
                v.placa as vehiculo_placa,
                v.capacidad as vehiculo_capacidad,
                v.modelo as vehiculo_modelo,
                tr.id as transporte_id,
                tr.nombre as transporte_nombre,
                tr.ruc as transporte_ruc,
                tr.telefono as transporte_telefono,
                c.nro_brevete as conductor_brevete,
                p.nombre as conductor_nombre,
                p.apellidos as conductor_apellidos,
                p.telefono as conductor_telefono
            FROM file_vehiculos fv
            INNER JOIN vehiculos v ON v.id = fv.id_vehiculo
            LEFT JOIN transportes tr ON tr.id = v.id_transporte
            LEFT JOIN conductores c ON c.id = fv.id_conductor
            LEFT JOIN personas p ON p.id = c.id_persona
            WHERE fv.id_file_tour = $1
            ORDER BY fv.created_at ASC
        "#)
        .bind::<Integer, _>(id_file_tour);
        
        query.load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error obteniendo vehículos con detalles: {}", e)))
    }
}

// ==================== FILE TOUR REPOSITORY ====================

pub struct PostgresFileTourRepository {
    pool: DatabasePool,
}

impl PostgresFileTourRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileTourRepositoryPort for PostgresFileTourRepository {
    #[instrument(skip(self, tours))]
    async fn add_many(&self, id_file: i32, tours: Vec<FileTourInputData>, created_by: Option<i32>) -> Result<Vec<FileTourModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Insertar cada tour individualmente para evitar problemas de lifetime
        let mut results = Vec::new();
        for data in tours {
            let new_record = NewFileTourModel {
                id_file,
                id_tour: data.id_tour,
                orden: data.orden,
                precio_aplicado: data.precio_aplicado,
                notas: data.notas.as_deref(),
                created_by,
                fecha_tour: data.fecha_tour,
                turno_tour: data.turno_tour.as_deref(),
                lugar_recojo: data.lugar_recojo.as_deref(),
                hora_recojo: data.hora_recojo,
                status: data.status.as_deref().unwrap_or("pendiente"),
                geo_recojo: data.geo_recojo.clone(),
            };
            
            let result = diesel::insert_into(file_tours::table)
                .values(&new_record)
                .returning(FileTourModel::as_returning())
                .get_result(&mut conn)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?;
            
            results.push(result);
        }
        
        info!("{} tours asignados a file {}", results.len(), id_file);
        Ok(results)
    }
    
    async fn remove_by_file(&self, id_file: i32) -> Result<usize, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(file_tours::table.filter(file_tours::id_file.eq(id_file)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Eliminados {} tours del file {}", affected, id_file);
        Ok(affected)
    }
    
    async fn find_by_file_with_tour(&self, id_file: i32) -> Result<Vec<FileTourWithTourModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // INNER JOIN completo entre file_tours y tours
        let results: Vec<(FileTourModel, (String, Option<String>, Option<String>, BigDecimal, Option<i32>, Option<String>, bool, Option<serde_json::Value>, Option<serde_json::Value>, Option<serde_json::Value>))> = file_tours::table
            .inner_join(tours::table.on(tours::id.eq(file_tours::id_tour)))
            .filter(file_tours::id_file.eq(id_file))
            .order(file_tours::orden.asc())
            .select((
                FileTourModel::as_select(),
                (
                    tours::nombre,
                    tours::lugar_inicio,
                    tours::lugar_fin,
                    tours::precio_base,
                    tours::duracion_dias,
                    tours::tipo_tour,
                    tours::is_active,
                    tours::geo_inicio,
                    tours::geo_fin,
                    tours::geo_ruta,
                ),
            ))
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al cargar tours con detalle: {}", e)))?;
        
        // Convertir a modelo con tour info completa (incluyendo fecha_tour y campos de recojo)
        let with_tour: Vec<FileTourWithTourModel> = results.into_iter().map(|(ft, (nombre, lugar_inicio, lugar_fin, precio, duracion, tipo, is_active, geo_inicio, geo_fin, geo_ruta))| {
            FileTourWithTourModel {
                id: ft.id,
                id_file: ft.id_file,
                id_tour: ft.id_tour,
                orden: ft.orden,
                precio_aplicado: ft.precio_aplicado,
                notas: ft.notas,
                created_at: ft.created_at,
                created_by: ft.created_by,
                fecha_tour: ft.fecha_tour,
                turno_tour: ft.turno_tour,
                lugar_recojo: ft.lugar_recojo,
                hora_recojo: ft.hora_recojo,
                status: ft.status,
                geo_recojo: ft.geo_recojo,
                tour_nombre: nombre,
                tour_lugar_inicio: lugar_inicio,
                tour_lugar_fin: lugar_fin,
                tour_precio_base: precio,
                tour_duracion_dias: duracion,
                tour_tipo: tipo,
                tour_is_active: is_active,
                tour_geo_inicio: geo_inicio,
                tour_geo_fin: geo_fin,
                tour_geo_ruta: geo_ruta,
            }
        }).collect();
        
        Ok(with_tour)
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<FileTourModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_tours::table
            .filter(file_tours::id.eq(id))
            .select(FileTourModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, status: &str) -> Result<FileTourModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_tours::table.filter(file_tours::id.eq(id)))
            .set(file_tours::status.eq(status))
            .returning(FileTourModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando status de file_tour: {}", e)))?;
        
        info!("Status de file_tour {} actualizado a '{}'", id, status);
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn update_hora_recojo(&self, id: i32, hora_recojo: Option<chrono::NaiveTime>) -> Result<FileTourModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_tours::table.filter(file_tours::id.eq(id)))
            .set(file_tours::hora_recojo.eq(hora_recojo))
            .returning(FileTourModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando hora_recojo de file_tour: {}", e)))?;
        
        info!("Hora recojo de file_tour {} actualizada a '{:?}'", id, hora_recojo);
        Ok(result)
    }
    
    #[instrument(skip(self))]
    async fn update_recojo(&self, id: i32, hora_recojo: Option<chrono::NaiveTime>, lugar_recojo: Option<String>, geo_recojo: Option<serde_json::Value>) -> Result<FileTourModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(file_tours::table.filter(file_tours::id.eq(id)))
            .set((
                file_tours::hora_recojo.eq(hora_recojo),
                file_tours::lugar_recojo.eq(lugar_recojo.as_deref()),
                file_tours::geo_recojo.eq(geo_recojo.as_ref()),
            ))
            .returning(FileTourModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando recojo de file_tour: {}", e)))?;
        
        info!("Recojo de file_tour {} actualizado: hora={:?}, lugar={:?}, geo={}", id, hora_recojo, lugar_recojo, geo_recojo.is_some());
        Ok(result)
    }
}
