use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::dtos::{CreateTarifaRequest, UpdateTarifaRequest, TarifaResponse};
use crate::application::ports::TarifaRepositoryPort;
use crate::domain::errors::ApplicationError;

pub struct TarifaService {
    tarifa_repository: Arc<dyn TarifaRepositoryPort>,
}

impl TarifaService {
    pub fn new(tarifa_repository: Arc<dyn TarifaRepositoryPort>) -> Self {
        Self { tarifa_repository }
    }

    /// Listar tarifas de un tour
    #[instrument(skip(self))]
    pub async fn get_tarifas_by_tour(&self, id_tour: i32) -> Result<Vec<TarifaResponse>, ApplicationError> {
        let tarifas = self.tarifa_repository.find_by_tour(id_tour).await?;
        info!("{} tarifas encontradas para tour {}", tarifas.len(), id_tour);
        Ok(tarifas.into_iter().map(Into::into).collect())
    }

    /// Obtener tarifa por ID
    #[instrument(skip(self))]
    pub async fn get_tarifa(&self, id: i32) -> Result<TarifaResponse, ApplicationError> {
        let tarifa = self.tarifa_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tarifa {} no encontrada", id)))?;
        Ok(tarifa.into())
    }

    /// Obtener tarifa por tour y tipo de entidad
    #[instrument(skip(self))]
    pub async fn get_tarifa_by_tour_and_tipo(&self, id_tour: i32, tipo_entidad: &str) -> Result<Option<TarifaResponse>, ApplicationError> {
        let tarifa = self.tarifa_repository.find_by_tour_and_tipo(id_tour, tipo_entidad).await?;
        Ok(tarifa.map(Into::into))
    }

    /// Crear una nueva tarifa
    #[instrument(skip(self, request))]
    pub async fn create_tarifa(&self, request: CreateTarifaRequest, created_by: Option<i32>) -> Result<TarifaResponse, ApplicationError> {
        // Verificar que no exista ya una tarifa para este tour + tipo_entidad
        let existing = self.tarifa_repository
            .find_by_tour_and_tipo(request.id_tour, &request.tipo_entidad)
            .await?;
        if existing.is_some() {
            return Err(ApplicationError::Conflict(
                format!("Ya existe una tarifa para tour {} con tipo_entidad '{}'", request.id_tour, request.tipo_entidad)
            ));
        }

        let entity = request.into_entity(created_by);
        let tarifa = self.tarifa_repository.create(&entity).await?;
        info!("Tarifa creada: tour={} tipo={}", tarifa.id_tour, tarifa.tipo_entidad);
        Ok(tarifa.into())
    }

    /// Actualizar una tarifa existente
    #[instrument(skip(self, request))]
    pub async fn update_tarifa(&self, id: i32, request: UpdateTarifaRequest, updated_by: Option<i32>) -> Result<TarifaResponse, ApplicationError> {
        let tarifa = self.tarifa_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tarifa {} no encontrada", id)))?;

        let updated = request.apply_to(tarifa, updated_by);
        let result = self.tarifa_repository.update(&updated).await?;
        info!("Tarifa {} actualizada", id);
        Ok(result.into())
    }

    /// Eliminar una tarifa
    #[instrument(skip(self))]
    pub async fn delete_tarifa(&self, id: i32) -> Result<bool, ApplicationError> {
        let deleted = self.tarifa_repository.delete(id).await?;
        if deleted {
            info!("Tarifa {} eliminada", id);
        }
        Ok(deleted)
    }

    /// Eliminar todas las tarifas de un tour
    #[instrument(skip(self))]
    pub async fn delete_tarifas_by_tour(&self, id_tour: i32) -> Result<i64, ApplicationError> {
        let count = self.tarifa_repository.delete_by_tour(id_tour).await?;
        info!("{} tarifas eliminadas para tour {}", count, id_tour);
        Ok(count)
    }
}
