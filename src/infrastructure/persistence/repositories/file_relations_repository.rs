use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{
        FileEntradaModel, NewFileEntradaModel,
        FileGuiaModel, NewFileGuiaModel,
        FilePasajeroModel, FilePasajeroWithPersonaModel, NewFilePasajeroModel,
        FileRestauranteModel, NewFileRestauranteModel,
        FileVehiculoModel, NewFileVehiculoModel,
    },
    schema::{file_entradas, file_guias, file_pasajeros, file_restaurantes, file_vehiculos},
};

// ==================== MODELOS EXTENDIDOS ====================

/// Modelo para file_vehiculo con datos extendidos de file, tour, agencia, vehiculo y conductor
#[derive(Debug, Clone, QueryableByName)]
pub struct FileVehiculoWithDetailsModel {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub id_file: i32,
    #[diesel(sql_type = Integer)]
    pub id_vehiculo: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub id_conductor: Option<i32>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Integer)]
    pub capacidad_asignada: i32,
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

// ==================== TRAITS (PORTS) ====================

#[async_trait]
pub trait FileEntradaRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, id_entrada: i32, cantidad: i32, created_by: Option<i32>) -> Result<FileEntradaModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileEntradaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileEntradaModel>, ApplicationError>;
}

#[async_trait]
pub trait FileGuiaRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, id_guia: i32, rol: Option<&str>, created_by: Option<i32>) -> Result<FileGuiaModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileGuiaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileGuiaModel>, ApplicationError>;
    async fn is_guia_assigned(&self, id_guia: i32, id_file: i32) -> Result<bool, ApplicationError>;
}

#[async_trait]
pub trait FilePasajeroRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, id_persona: i32, asiento: Option<&str>, tipo_pasajero: Option<&str>, nacionalidad: Option<&str>, notas: Option<&str>, created_by: Option<i32>) -> Result<FilePasajeroModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_with_persona(&self, id_file: i32) -> Result<Vec<FilePasajeroWithPersonaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FilePasajeroModel>, ApplicationError>;
    async fn count_by_file(&self, id_file: i32) -> Result<i64, ApplicationError>;
}

#[async_trait]
pub trait FileRestauranteRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, id_restaurante: i32, tipo_servicio: Option<&str>, dia: Option<i32>, created_by: Option<i32>) -> Result<FileRestauranteModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileRestauranteModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileRestauranteModel>, ApplicationError>;
}

#[async_trait]
pub trait FileVehiculoRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, id_vehiculo: i32, id_conductor: Option<i32>, capacidad_asignada: i32, created_by: Option<i32>) -> Result<FileVehiculoModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileVehiculoModel>, ApplicationError>;
    async fn find_all_with_details(&self) -> Result<Vec<FileVehiculoWithDetailsModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileVehiculoModel>, ApplicationError>;
    async fn find_files_by_vehiculo(&self, id_vehiculo: i32) -> Result<Vec<i32>, ApplicationError>;
    async fn is_vehiculo_assigned(&self, id_vehiculo: i32, id_file: i32) -> Result<bool, ApplicationError>;
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
    async fn add(&self, id_file: i32, id_entrada: i32, cantidad: i32, created_by: Option<i32>) -> Result<FileEntradaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileEntradaModel {
            id_file,
            id_entrada,
            cantidad,
            created_by,
        };
        
        let result = diesel::insert_into(file_entradas::table)
            .values(&new_record)
            .returning(FileEntradaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Entrada asignada a file: file={}, entrada={}, cantidad={}", id_file, id_entrada, cantidad);
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
    
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileEntradaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_entradas::table
            .filter(file_entradas::id_file.eq(id_file))
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
    async fn add(&self, id_file: i32, id_guia: i32, rol: Option<&str>, created_by: Option<i32>) -> Result<FileGuiaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileGuiaModel {
            id_file,
            id_guia,
            rol,
            created_by,
        };
        
        let result = diesel::insert_into(file_guias::table)
            .values(&new_record)
            .returning(FileGuiaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Guía asignado a file: file={}, guia={}", id_file, id_guia);
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
    
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileGuiaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_guias::table
            .filter(file_guias::id_file.eq(id_file))
            .select(FileGuiaModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
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
    
    async fn is_guia_assigned(&self, id_guia: i32, id_file: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = file_guias::table
            .filter(file_guias::id_guia.eq(id_guia))
            .filter(file_guias::id_file.eq(id_file))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
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
    async fn add(&self, id_file: i32, id_persona: i32, asiento: Option<&str>, tipo_pasajero: Option<&str>, nacionalidad: Option<&str>, notas: Option<&str>, created_by: Option<i32>) -> Result<FilePasajeroModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFilePasajeroModel {
            id_file,
            id_persona,
            asiento,
            tipo_pasajero,
            notas,
            created_by,
            nacionalidad,
        };
        
        let result = diesel::insert_into(file_pasajeros::table)
            .values(&new_record)
            .returning(FilePasajeroModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Pasajero agregado a file: file={}, persona={}", id_file, id_persona);
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
                p.nombre as pasajero_nombre,
                p.apellidos as pasajero_apellidos,
                p.nro_documento as pasajero_documento
            FROM file_pasajeros fp
            INNER JOIN personas p ON p.id = fp.id_persona
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
    async fn add(&self, id_file: i32, id_restaurante: i32, tipo_servicio: Option<&str>, dia: Option<i32>, created_by: Option<i32>) -> Result<FileRestauranteModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileRestauranteModel {
            id_file,
            id_restaurante,
            tipo_servicio,
            dia,
            created_by,
        };
        
        let result = diesel::insert_into(file_restaurantes::table)
            .values(&new_record)
            .returning(FileRestauranteModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Restaurante asignado a file: file={}, restaurante={}", id_file, id_restaurante);
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
    
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileRestauranteModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_restaurantes::table
            .filter(file_restaurantes::id_file.eq(id_file))
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
    async fn add(&self, id_file: i32, id_vehiculo: i32, id_conductor: Option<i32>, capacidad_asignada: i32, created_by: Option<i32>) -> Result<FileVehiculoModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let new_record = NewFileVehiculoModel {
            id_file,
            id_vehiculo,
            id_conductor,
            capacidad_asignada,
            created_by,
        };
        
        let result = diesel::insert_into(file_vehiculos::table)
            .values(&new_record)
            .returning(FileVehiculoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Vehículo asignado a file: file={}, vehiculo={}, conductor={:?}, capacidad={}", id_file, id_vehiculo, id_conductor, capacidad_asignada);
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
    
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileVehiculoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        file_vehiculos::table
            .filter(file_vehiculos::id_file.eq(id_file))
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
                fv.id_file,
                fv.id_vehiculo,
                fv.id_conductor,
                fv.created_at,
                fv.capacidad_asignada,
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
            INNER JOIN files f ON f.id = fv.id_file
            INNER JOIN tours t ON t.id = f.id_tour
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
        
        info!("✅ Encontrados {} file_vehiculos con detalles", results.len());
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
            .select(file_vehiculos::id_file)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results)
    }
    
    async fn is_vehiculo_assigned(&self, id_vehiculo: i32, id_file: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = file_vehiculos::table
            .filter(file_vehiculos::id_vehiculo.eq(id_vehiculo))
            .filter(file_vehiculos::id_file.eq(id_file))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
}
