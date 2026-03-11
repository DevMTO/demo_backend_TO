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
use serde_json::Value as JsonValue;
use crate::domain::errors::ApplicationError;

// Importamos los modelos que necesitamos
use crate::infrastructure::persistence::models::{
    FileEntradaModel, FileGuiaModel, FileGuiaWithPersonaModel, FilePasajeroModel, 
    FilePasajeroWithPersonaModel, FileRestauranteModel, 
    FileVehiculoModel, FileVehiculoWithPersonaModel, FileTourModel, FileTourWithTourModel,
    file_pasajero_model::UpdateFilePasajeroModel,
    file_vehiculo_model::UpdateFileVehiculoModel,
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
    /// Coordenadas de geolocalización del punto de recojo
    pub geo_recojo: Option<JsonValue>,
    /// Cantidad de pasajeros específicos para este tour
    pub nro_pasajeros: Option<i32>,
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
    /// Transfiere una file_entrada a otro file_tour
    async fn transfer_to_file_tour(&self, id: i32, new_id_file_tour: i32) -> Result<FileEntradaModel, ApplicationError>;
}

/// Repositorio para file_guias (vinculado a file_tours)
#[async_trait]
pub trait FileGuiaRepositoryPort: Send + Sync {
    async fn add(&self, id_file_tour: i32, id_guia: i32, rol: Option<&str>, created_by: Option<i32>) -> Result<FileGuiaModel, ApplicationError>;
    async fn remove(&self, id: i32) -> Result<bool, ApplicationError>;
    /// Busca guías de un file_tour con información completa del guía y persona (JOIN)
    async fn find_by_file_tour_with_persona(&self, id_file_tour: i32) -> Result<Vec<FileGuiaWithPersonaModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileGuiaModel>, ApplicationError>;
    async fn is_guia_assigned(&self, id_guia: i32, id_file_tour: i32) -> Result<bool, ApplicationError>;
    /// Actualiza el status de una file_guia (permite 'pendiente')
    async fn update_status(&self, id: i32, status: &str) -> Result<FileGuiaModel, ApplicationError>;
    /// Actualiza id_guia y/o id_file_tour de una file_guia (PATCH parcial)
    async fn update(&self, id: i32, data: crate::infrastructure::persistence::models::file_guia_model::UpdateFileGuiaModel) -> Result<FileGuiaModel, ApplicationError>;
}

#[async_trait]
pub trait FilePasajeroRepositoryPort: Send + Sync {
    /// Agrega un pasajero a un file
    /// - id_persona es opcional para permitir pasajeros anónimos
    /// - edad es opcional
    #[allow(clippy::too_many_arguments)]
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
    /// Actualiza los campos de un file_vehiculo (vehículo, conductor, capacidad, status)
    async fn update(&self, id: i32, data: UpdateFileVehiculoModel) -> Result<FileVehiculoModel, ApplicationError>;
}

/// Repositorio para file_tours
#[async_trait]
pub trait FileTourRepositoryPort: Send + Sync {
    async fn add_many(&self, id_file: i32, tours: Vec<FileTourInputData>, created_by: Option<i32>) -> Result<Vec<FileTourModel>, ApplicationError>;
    async fn remove_by_file(&self, id_file: i32) -> Result<usize, ApplicationError>;
    /// Busca tours de un file con información del tour (INNER JOIN)
    async fn find_by_file_with_tour(&self, id_file: i32) -> Result<Vec<FileTourWithTourModel>, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<FileTourModel>, ApplicationError>;
    /// Actualiza un file_tour completo
    async fn update(&self, file_tour: &FileTourModel) -> Result<FileTourModel, ApplicationError>;
    /// Actualiza el status de un file_tour
    async fn update_status(&self, id: i32, status: &str) -> Result<FileTourModel, ApplicationError>;
    /// Actualiza la hora de recojo de un file_tour
    async fn update_hora_recojo(&self, id: i32, hora_recojo: Option<chrono::NaiveTime>) -> Result<FileTourModel, ApplicationError>;
    /// Actualiza la información de recojo de un file_tour (hora, lugar y/o geo)
    async fn update_recojo(&self, id: i32, hora_recojo: Option<chrono::NaiveTime>, lugar_recojo: Option<String>, geo_recojo: Option<serde_json::Value>) -> Result<FileTourModel, ApplicationError>;
    /// Actualiza el precio_aplicado de un file_tour
    async fn update_precio_aplicado(&self, id: i32, precio_aplicado: Option<BigDecimal>) -> Result<FileTourModel, ApplicationError>;
}
