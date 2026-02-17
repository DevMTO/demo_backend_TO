//! Servicio para la gestión de estados de FileTour
//!
//! Este servicio maneja la lógica de negocio para:
//! - Actualizar el estado de un FileTour
//! - Propagar cambios de estado a entidades relacionadas (guías, vehículos, restaurantes)
//! - Definir reglas de cascada para estados específicos

use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::ports::{
    FileTourRepositoryPort, FileGuiaRepositoryPort,
    FileVehiculoRepositoryPort, FileRestauranteRepositoryPort,
};
use crate::domain::errors::ApplicationError;

/// Statuses que disparan cascada a entidades relacionadas (guías, vehículos, restaurantes)
const CASCADE_CANCELING_STATUSES: &[&str] = &["asignado", "cancelado"];

/// Resultado de una actualización de estado de FileTour con información de cascada
#[derive(Debug, Clone)]
pub struct UpdateFileTourStatusResult {
    pub old_status: String,
    pub new_status: String,
    pub guias_actualizados: usize,
    pub vehiculos_actualizados: usize,
    pub restaurantes_actualizados: usize,
}

impl UpdateFileTourStatusResult {
    /// Verifica si hubo actualizaciones en entidades relacionadas
    pub fn has_cascade_updates(&self) -> bool {
        self.guias_actualizados > 0
            || self.vehiculos_actualizados > 0
            || self.restaurantes_actualizados > 0
    }

    /// Genera un mensaje descriptivo del resultado
    pub fn to_message(&self) -> String {
        if self.has_cascade_updates() {
            format!(
                "Status de tour actualizado de '{}' a '{}'. Guias: {}, Vehiculos: {}, Restaurantes: {}",
                self.old_status,
                self.new_status,
                self.guias_actualizados,
                self.vehiculos_actualizados,
                self.restaurantes_actualizados
            )
        } else {
            format!(
                "Status de tour actualizado de '{}' a '{}'",
                self.old_status, self.new_status
            )
        }
    }
}

/// Servicio para la gestión de estados de FileTour
pub struct FileTourStatusService {
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
}

impl FileTourStatusService {
    pub fn new(
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
        file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
        file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    ) -> Self {
        Self {
            file_tour_repository,
            file_guia_repository,
            file_vehiculo_repository,
            file_restaurante_repository,
        }
    }

    /// Actualiza el estado de un FileTour y propaga la cascada si corresponde
    ///
    /// # Arguments
    /// * `file_tour_id` - ID del FileTour a actualizar
    /// * `new_status` - Nuevo estado a asignar
    ///
    /// # Returns
    /// * `UpdateFileTourStatusResult` con información del resultado y las entidades actualizadas
    #[instrument(skip(self))]
    pub async fn update_status(
        &self,
        file_tour_id: i32,
        new_status: &str,
    ) -> Result<UpdateFileTourStatusResult, ApplicationError> {
        // Obtener registro actual
        let current = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        let old_status = current.status.clone();

        // Actualizar status del FileTour
        self.file_tour_repository
            .update_status(file_tour_id, new_status)
            .await?;

        info!(
            "FileTour {} actualizado de '{}' a '{}'",
            file_tour_id, old_status, new_status
        );

        // Verificar si debe aplicarse cascada
        let should_cascade = CASCADE_CANCELING_STATUSES.contains(&new_status);

        let mut result = UpdateFileTourStatusResult {
            old_status,
            new_status: new_status.to_string(),
            guias_actualizados: 0,
            vehiculos_actualizados: 0,
            restaurantes_actualizados: 0,
        };

        if should_cascade {
            result = self
                .apply_cascade(file_tour_id, new_status, result)
                .await?;
        }

        Ok(result)
    }

    /// Aplica la cascada de estado a entidades relacionadas
    ///
    /// Solo actualiza entidades que no estén en estados finales (cancelado, completado)
    #[instrument(skip(self))]
    async fn apply_cascade(
        &self,
        file_tour_id: i32,
        new_status: &str,
        mut result: UpdateFileTourStatusResult,
    ) -> Result<UpdateFileTourStatusResult, ApplicationError> {
        // Actualizar file_guias
        let guias = self.file_guia_repository
            .find_by_file_tour_with_persona(file_tour_id)
            .await?;

        for guia in guias {
            if guia.status != "cancelado" && guia.status != "completado" {
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
            if vehiculo.status != "cancelado" && vehiculo.status != "completado" {
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
            if restaurante.status != "cancelado" && restaurante.status != "completado" {
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

        info!(
            "Cascada completada para FileTour {}: {} guías, {} vehículos, {} restaurantes actualizados",
            file_tour_id,
            result.guias_actualizados,
            result.vehiculos_actualizados,
            result.restaurantes_actualizados
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_cascade() {
        assert!(CASCADE_CANCELING_STATUSES.contains(&"cancelado"));
        assert!(!CASCADE_CANCELING_STATUSES.contains(&"confirmado"));
        assert!(!CASCADE_CANCELING_STATUSES.contains(&"asignado"));
        assert!(!CASCADE_CANCELING_STATUSES.contains(&"pendiente"));
    }

    #[test]
    fn test_result_message_without_cascade() {
        let result = UpdateFileTourStatusResult {
            old_status: "confirmado".to_string(),
            new_status: "asignado".to_string(),
            guias_actualizados: 0,
            vehiculos_actualizados: 0,
            restaurantes_actualizados: 0,
        };

        assert_eq!(
            result.to_message(),
            "Status de tour actualizado de 'confirmado' a 'asignado'"
        );
    }

    #[test]
    fn test_result_message_with_cascade() {
        let result = UpdateFileTourStatusResult {
            old_status: "confirmado".to_string(),
            new_status: "cancelado".to_string(),
            guias_actualizados: 2,
            vehiculos_actualizados: 1,
            restaurantes_actualizados: 3,
        };

        assert_eq!(
            result.to_message(),
            "Status de tour actualizado de 'confirmado' a 'cancelado'. Guias: 2, Vehiculos: 1, Restaurantes: 3"
        );
    }
}
