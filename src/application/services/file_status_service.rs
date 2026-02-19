//! Servicio para la gestión de estados de File
//!
//! Este servicio maneja la lógica de negocio para:
//! - Actualizar el estado de un File
//! - Propagar cambios de estado a entidades relacionadas (guías, vehículos, restaurantes, entradas)
//! - Definir reglas de cascada para estados específicos

use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::ports::{
    FileTourRepositoryPort, FileGuiaRepositoryPort, FileVehiculoRepositoryPort, FileRestauranteRepositoryPort, FileEntradaRepositoryPort,
};
use crate::domain::errors::ApplicationError;

/// Statuses que disparan cascada a entidades relacionadas (guías, vehículos, restaurantes, entradas)
const CASCADE_STATUSES: &[&str] = &["asignado", "cancelado", "no_show"];

/// Statuses finales que no deben ser actualizados por cascada
const FINAL_STATUSES: &[&str] = &["cancelado", "completado"];

/// Resultado de una actualización de estado de FileTour con información de cascada
#[derive(Debug, Clone)]
pub struct UpdateFileStatusResult {
    pub old_status: String,
    pub new_status: String,
    pub guias_actualizados: usize,
    pub vehiculos_actualizados: usize,
    pub restaurantes_actualizados: usize,
    pub entradas_actualizadas: usize,
}

impl UpdateFileStatusResult {
    /// Verifica si hubo actualizaciones en entidades relacionadas
    pub fn has_cascade_updates(&self) -> bool {
        self.guias_actualizados > 0
            || self.vehiculos_actualizados > 0
            || self.restaurantes_actualizados > 0
            || self.entradas_actualizadas > 0
    }

    /// Genera un mensaje descriptivo del resultado
    pub fn to_message(&self) -> String {
        if self.has_cascade_updates() {
            format!(
                "Status de tour actualizado de '{}' a '{}'. Guias: {}, Vehiculos: {}, Restaurantes: {}, Entradas: {}",
                self.old_status,
                self.new_status,
                self.guias_actualizados,
                self.vehiculos_actualizados,
                self.restaurantes_actualizados,
                self.entradas_actualizadas
            )
        } else {
            format!(
                "Status de tour actualizado de '{}' a '{}'",
                self.old_status, self.new_status
            )
        }
    }
}

/// Servicio para la gestión de estados de File
pub struct FileStatusService {
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
}

impl FileStatusService {
    pub fn new(
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
        file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
        file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
        file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    ) -> Self {
        Self {
            file_tour_repository,
            file_guia_repository,
            file_vehiculo_repository,
            file_restaurante_repository,
            file_entrada_repository,
        }
    }

    /// Actualiza el estado de un FileTour y propaga la cascada a entidades relacionadas
    ///
    /// # Arguments
    /// * `file_tour_id` - ID del FileTour a actualizar
    /// * `new_status` - Nuevo estado a asignar
    ///
    /// # Returns
    /// * `UpdateFileStatusResult` con información del resultado y las entidades actualizadas
    #[instrument(skip(self))]
    pub async fn update_file_tour_status(
        &self,
        file_tour_id: i32,
        new_status: &str,
    ) -> Result<UpdateFileStatusResult, ApplicationError> {
        let current = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        let old_status = current.status.clone();

        self.file_tour_repository
            .update_status(file_tour_id, new_status)
            .await?;

        info!(
            "FileTour {} actualizado de '{}' a '{}'",
            file_tour_id, old_status, new_status
        );

        if CASCADE_STATUSES.contains(&new_status) {
            let mut result = UpdateFileStatusResult {
                old_status,
                new_status: new_status.to_string(),
                guias_actualizados: 0,
                vehiculos_actualizados: 0,
                restaurantes_actualizados: 0,
                entradas_actualizadas: 0,
            };

            result = self
                .propagate_status_to_relations(file_tour_id, new_status, result)
                .await?;

            Ok(result)
        } else {
            Ok(UpdateFileStatusResult {
                old_status,
                new_status: new_status.to_string(),
                guias_actualizados: 0,
                vehiculos_actualizados: 0,
                restaurantes_actualizados: 0,
                entradas_actualizadas: 0,
            })
        }
    }

    /// Propaga el estado a entidades relacionadas (file_guias, file_vehiculos, file_restaurantes, file_entradas)
    ///
    /// Solo actualiza entidades que no estén en estados finales (cancelado, completado)
    #[instrument(skip(self))]
    async fn propagate_status_to_relations(
        &self,
        file_tour_id: i32,
        new_status: &str,
        mut result: UpdateFileStatusResult,
    ) -> Result<UpdateFileStatusResult, ApplicationError> {
        if !CASCADE_STATUSES.contains(&new_status) {
            return Ok(result);
        }

        // Actualizar file_guias
        let guias = self.file_guia_repository
            .find_by_file_tour_with_persona(file_tour_id)
            .await?;

        for guia in guias {
            if !FINAL_STATUSES.contains(&guia.status.as_str()) {
                self.file_guia_repository
                    .update_status(guia.id, new_status)
                    .await?;
                result.guias_actualizados += 1;
                info!(
                    "FileGuia {} actualizado a '{}' por cascada de FileTour {}",
                    guia.id, new_status, file_tour_id
                );
            }
        }

        // Actualizar file_vehiculos
        let vehiculos = self.file_vehiculo_repository
            .find_by_file_tour_with_persona(file_tour_id)
            .await?;

        for vehiculo in vehiculos {
            if !FINAL_STATUSES.contains(&vehiculo.status.as_str()) {
                self.file_vehiculo_repository
                    .update_status(vehiculo.id, new_status)
                    .await?;
                result.vehiculos_actualizados += 1;
                info!(
                    "FileVehiculo {} actualizado a '{}' por cascada de FileTour {}",
                    vehiculo.id, new_status, file_tour_id
                );
            }
        }

        // Actualizar file_restaurantes
        let restaurantes = self.file_restaurante_repository
            .find_by_file_tour(file_tour_id)
            .await?;

        for restaurante in restaurantes {
            if !FINAL_STATUSES.contains(&restaurante.status.as_str()) {
                self.file_restaurante_repository
                    .update_status(restaurante.id, new_status)
                    .await?;
                result.restaurantes_actualizados += 1;
                info!(
                    "FileRestaurante {} actualizado a '{}' por cascada de FileTour {}",
                    restaurante.id, new_status, file_tour_id
                );
            }
        }

        // Actualizar file_entradas
        let entradas = self.file_entrada_repository
            .find_by_file_tour(file_tour_id)
            .await?;

        for entrada in entradas {
            if !FINAL_STATUSES.contains(&entrada.status.as_str()) {
                self.file_entrada_repository
                    .update_status(entrada.id, new_status)
                    .await?;
                result.entradas_actualizadas += 1;
                info!(
                    "FileEntrada {} actualizado a '{}' por cascada de FileTour {}",
                    entrada.id, new_status, file_tour_id
                );
            }
        }

        info!(
            "Cascada completada para FileTour {}: {} guías, {} vehículos, {} restaurantes, {} entradas actualizadas",
            file_tour_id,
            result.guias_actualizados,
            result.vehiculos_actualizados,
            result.restaurantes_actualizados,
            result.entradas_actualizadas
        );

        Ok(result)
    }
}
