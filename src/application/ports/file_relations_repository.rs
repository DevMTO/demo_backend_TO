//! Ports (traits) para repositorios de relaciones de files
//! 
//! Define las interfaces para:
//! - FileEntradaRepositoryPort
//! - FileGuiaRepositoryPort
//! - FilePasajeroRepositoryPort
//! - FileRestauranteRepositoryPort
//! - FileVehiculoRepositoryPort
//! - FileTourRepositoryPort

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveTime};
use crate::domain::errors::ApplicationError;

// Importamos los modelos que necesitamos
use crate::infrastructure::persistence::models::{
    FileEntradaModel, FileGuiaModel, FileGuiaWithPersonaModel, FilePasajeroModel, 
    FilePasajeroWithPersonaModel, FileRestauranteModel, 
    FileVehiculoModel, FileVehiculoWithPersonaModel, FileTourModel, FileTourWithTourModel,
    file_pasajero_model::UpdateFilePasajeroModel,
};
// FileVehiculoWithDetailsModel viene de repositories
use crate::infrastructure::persistence::repositories::FileVehiculoWithDetailsModel;

/// Datos de entrada para crear un FileTour
#[derive(Debug, Clone)]
pub struct FileTourInputData {
    pub id_tour: i32,
    pub orden: i32,
    pub precio_aplicado: Option<BigDecimal>,
    pub notas: Option<String>,
    pub fecha_tour: Option<NaiveDate>,
    pub turno_tour: Option<String>,
    pub lugar_recojo: Option<String>,
    pub hora_recojo: Option<NaiveTime>,
    /// Estado del file_tour (default: pendiente)
    pub status: Option<String>,
}

/// Repositorio para file_entradas (vinculado a file_tours)
#[async_trait]
pub trait FileEntradaRepositoryPort: Send + Sync {
    async fn add(&self, id_file_tour: i32, id_entrada: i32, cantidad: i32, id_entrada_precio: Option<i32>, created_by: Option<i32>) -> Result<FileEntradaModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileEntradaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileEntradaModel>, ApplicationError>;
    /// Actualiza el status de una file_entrada
    async fn update_status(&self, id: i32, status: &str) -> Result<FileEntradaModel, ApplicationError>;
}

/// Repositorio para file_guias (vinculado a file_tours)
#[async_trait]
pub trait FileGuiaRepositoryPort: Send + Sync {
    async fn add(&self, id_file_tour: i32, id_guia: i32, rol: Option<&str>, created_by: Option<i32>) -> Result<FileGuiaModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileGuiaModel>, ApplicationError>;
    /// Busca guías de un file_tour con información completa del guía y persona (JOIN)
    async fn find_by_file_tour_with_persona(&self, id_file_tour: i32) -> Result<Vec<FileGuiaWithPersonaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileGuiaModel>, ApplicationError>;
    async fn is_guia_assigned(&self, id_guia: i32, id_file_tour: i32) -> Result<bool, ApplicationError>;
    /// Actualiza el status de una file_guia (permite 'pendiente')
    async fn update_status(&self, id: i32, status: &str) -> Result<FileGuiaModel, ApplicationError>;
}

#[async_trait]
pub trait FilePasajeroRepositoryPort: Send + Sync {
    /// Agrega un pasajero a un file
    /// - id_persona es opcional para permitir pasajeros anónimos
    /// - edad es opcional
    async fn add(&self, id_file: i32, id_persona: Option<i32>, asiento: Option<&str>, tipo_pasajero: Option<&str>, nacionalidad: Option<&str>, notas: Option<&str>, edad: Option<i32>, created_by: Option<i32>) -> Result<FilePasajeroModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_with_persona(&self, id_file: i32) -> Result<Vec<FilePasajeroWithPersonaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FilePasajeroModel>, ApplicationError>;
    async fn count_by_file(&self, id_file: i32) -> Result<i64, ApplicationError>;
    /// Actualiza el status de un file_pasajero
    async fn update_status(&self, id: i32, status: &str) -> Result<FilePasajeroModel, ApplicationError>;
    /// Actualiza la información de un file_pasajero
    async fn update(&self, id: i32, data: UpdateFilePasajeroModel) -> Result<FilePasajeroModel, ApplicationError>;
}

/// Repositorio para file_restaurantes (vinculado a file_tours)
#[async_trait]
pub trait FileRestauranteRepositoryPort: Send + Sync {
    async fn add(&self, id_file_tour: i32, id_restaurante: i32, tipo_servicio: Option<&str>, precio: Option<BigDecimal>, created_by: Option<i32>) -> Result<FileRestauranteModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileRestauranteModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileRestauranteModel>, ApplicationError>;
    /// Actualiza el status de una file_restaurante
    async fn update_status(&self, id: i32, status: &str) -> Result<FileRestauranteModel, ApplicationError>;
}

/// Repositorio para file_vehiculos (vinculado a file_tours)
#[async_trait]
pub trait FileVehiculoRepositoryPort: Send + Sync {
    async fn add(&self, id_file_tour: i32, id_vehiculo: i32, id_conductor: Option<i32>, capacidad_asignada: i32, created_by: Option<i32>) -> Result<FileVehiculoModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<FileVehiculoModel>, ApplicationError>;
    /// Busca vehículos de un file_tour con información completa (vehículo, conductor, transporte, persona)
    async fn find_by_file_tour_with_persona(&self, id_file_tour: i32) -> Result<Vec<FileVehiculoWithPersonaModel>, ApplicationError>;
    async fn find_all_with_details(&self) -> Result<Vec<FileVehiculoWithDetailsModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileVehiculoModel>, ApplicationError>;
    async fn find_files_by_vehiculo(&self, id_vehiculo: i32) -> Result<Vec<i32>, ApplicationError>;
    async fn is_vehiculo_assigned(&self, id_vehiculo: i32, id_file_tour: i32) -> Result<bool, ApplicationError>;
    /// Actualiza el status de un file_vehiculo
    async fn update_status(&self, id: i32, status: &str) -> Result<FileVehiculoModel, ApplicationError>;
}

/// Repositorio para file_tours
#[async_trait]
pub trait FileTourRepositoryPort: Send + Sync {
    async fn add(&self, id_file: i32, data: FileTourInputData, created_by: Option<i32>) -> Result<FileTourModel, ApplicationError>;
    async fn add_many(&self, id_file: i32, tours: Vec<FileTourInputData>, created_by: Option<i32>) -> Result<Vec<FileTourModel>, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn remove_by_file(&self, id_file: i32) -> Result<usize, ApplicationError>;
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<FileTourModel>, ApplicationError>;
    /// Busca tours de un file con información del tour (INNER JOIN)
    async fn find_by_file_with_tour(&self, id_file: i32) -> Result<Vec<FileTourWithTourModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileTourModel>, ApplicationError>;
    async fn find_by_tour(&self, id_tour: i32) -> Result<Vec<FileTourModel>, ApplicationError>;
    async fn get_next_orden(&self, id_file: i32) -> Result<i32, ApplicationError>;
    /// Actualiza el status de un file_tour
    async fn update_status(&self, id: i32, status: &str) -> Result<FileTourModel, ApplicationError>;
}
