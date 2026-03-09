//! Servicio para la gestión de estados de File
//!
//! Este servicio maneja la lógica de negocio para:
//! - Actualizar el estado de un File
//! - Propagar cambios de estado a entidades relacionadas (guías, vehículos, restaurantes, entradas)
//! - Definir reglas de cascada para estados específicos

use std::sync::Arc;
use tracing::{info, instrument};

use crate::application::ports::{
    FileTourRepositoryPort, FileGuiaRepositoryPort, FileVehiculoRepositoryPort, FileRestauranteRepositoryPort, FileEntradaRepositoryPort, FileRepositoryPort,
    EntradaRepositoryPort, PagoProveedorRepositoryPort,
};
use crate::domain::errors::ApplicationError;


/// Statuses que disparan cascada a entidades relacionadas (guías, vehículos, restaurantes, entradas)
const CASCADE_STATUSES: &[&str] = &["asignado", "cancelado", "no_show"];

/// Statuses finales que no deben ser actualizados por cascada
const FINAL_STATUSES: &[&str] = &["cancelado", "completado", "no_show"];

/// Resultado de una actualización de estado de File con información de cascada
#[derive(Debug, Clone)]
pub struct UpdateFileStatusResult {
    pub old_status: String,
    pub new_status: String,
    pub guias_actualizados: usize,
    pub vehiculos_actualizados: usize,
    pub restaurantes_actualizados: usize,
    pub entradas_actualizadas: usize,
    pub pagos_proveedores_actualizados: usize,
}

impl UpdateFileStatusResult {
    /// Verifica si hubo actualizaciones en entidades relacionadas
    pub fn has_cascade_updates(&self) -> bool {
        self.guias_actualizados > 0
            || self.vehiculos_actualizados > 0
            || self.restaurantes_actualizados > 0
            || self.entradas_actualizadas > 0
            || self.pagos_proveedores_actualizados > 0
    }

    /// Genera un mensaje descriptivo del resultado
    pub fn to_message(&self) -> String {
        if self.has_cascade_updates() {
            format!(
                "Status de file actualizado de '{}' a '{}'. Guias: {}, Vehiculos: {}, Restaurantes: {}, Entradas: {}, Pagos proveedores: {}",
                self.old_status,
                self.new_status,
                self.guias_actualizados,
                self.vehiculos_actualizados,
                self.restaurantes_actualizados,
                self.entradas_actualizadas,
                self.pagos_proveedores_actualizados
            )
        } else {
            format!(
                "Status de file actualizado de '{}' a '{}'",
                self.old_status, self.new_status
            )
        }
    }
}

/// Servicio para la gestión de estados de File
pub struct FileStatusService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    entrada_repository: Arc<dyn EntradaRepositoryPort>,
    pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
}

impl FileStatusService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
        file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
        file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
        file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
        entrada_repository: Arc<dyn EntradaRepositoryPort>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
            file_guia_repository,
            file_vehiculo_repository,
            file_restaurante_repository,
            file_entrada_repository,
            entrada_repository,
            pago_proveedor_repository,
        }
    }

    /// Actualiza el estado de un File y propaga la cascada a entidades relacionadas
    ///
    /// # Arguments
    /// * `file_id` - ID del File a actualizar
    /// * `new_status` - Nuevo estado a asignar
    ///
    /// # Returns
    /// * `UpdateFileStatusResult` con información del resultado y las entidades actualizadas
    #[instrument(skip(self))]
    pub async fn update_file_status(
        &self,
        file_id: i32,
        new_status: &str,
    ) -> Result<UpdateFileStatusResult, ApplicationError> {
        let current = self.file_repository
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;

        let old_status = current.status.clone();

        self.file_repository
            .update_status(file_id, new_status)
            .await?;

        info!(
            "File {} actualizado de '{}' a '{}',",
            file_id, old_status, new_status
        );

        if CASCADE_STATUSES.contains(&new_status) {
            let mut result = UpdateFileStatusResult {
                old_status,
                new_status: new_status.to_string(),
                guias_actualizados: 0,
                vehiculos_actualizados: 0,
                restaurantes_actualizados: 0,
                entradas_actualizadas: 0,
                pagos_proveedores_actualizados: 0,
            };

            // Obtener todos los file_tours relacionados y actualizar file_tours
            let file_tours = self.file_tour_repository
                .find_by_file_with_tour(file_id)
                .await?;

            for file_tour in file_tours {
                if !FINAL_STATUSES.contains(&file_tour.status.as_str()) {
                    self.file_tour_repository
                        .update_status(file_tour.id, new_status)
                        .await?;
                    
                    info!(
                        "FileTour {} actualizado a '{}' por cascada de File {}",
                        file_tour.id, new_status, file_id
                    );

                    // Aplicar cascada a las entidades relacionadas de este file_tour
                    result = self
                        .propagate_status_to_relations(file_tour.id, new_status, result)
                        .await?;
                    
                    info!(
                        "Relaciones de FileTour {} actualizadas: guias={}, vehiculos={}, restaurantes={}, entradas={}, pagos_proveedores={}",
                        file_tour.id, result.guias_actualizados, result.vehiculos_actualizados, result.restaurantes_actualizados, result.entradas_actualizadas, result.pagos_proveedores_actualizados
                    );
                }
            }

            Ok(result)
        } else {
            Ok(UpdateFileStatusResult {
                old_status,
                new_status: new_status.to_string(),
                guias_actualizados: 0,
                vehiculos_actualizados: 0,
                restaurantes_actualizados: 0,
                entradas_actualizadas: 0,
                pagos_proveedores_actualizados: 0,
            })
        }
    }

    /// Propaga el estado a entidades relacionadas a un FileTour (guías, vehículos, restaurantes, entradas)
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
            .find_by_file_tour(file_tour_id)
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
                // Si es cancelación, verificar si la entrada es BT — no cancelar BT
                if new_status == "cancelado" {
                    if let Ok(Some(ent)) = self.entrada_repository.find_by_id(entrada.id_entrada).await {
                        if ent.boleto_turistico {
                            info!(
                                "FileEntrada {} es BT, se omite cancelación (se transfiere)",
                                entrada.id
                            );
                            continue;
                        }
                    }
                }
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

        result = self
            .propagate_status_to_pagos_proveedores(file_tour_id, new_status, result)
            .await?;

        info!(
            "Cascada completada para FileTour {}: {} entidades actualizadas",
            file_tour_id,
            result.guias_actualizados + result.vehiculos_actualizados + result.restaurantes_actualizados + result.entradas_actualizadas + result.pagos_proveedores_actualizados
        );

        Ok(result)
    }

    /// Propaga el estado a pagos de proveedores relacionados a un FileTour
    ///
    /// Solo actualiza entidades que no estén en estados finales (cancelado, completado)
    #[instrument(skip(self))]
    async fn propagate_status_to_pagos_proveedores(
        &self,
        file_tour_id: i32,
        new_status: &str,
        mut result: UpdateFileStatusResult,
    ) -> Result<UpdateFileStatusResult, ApplicationError> {
        if !CASCADE_STATUSES.contains(&new_status) {
            return Ok(result);
        }

        let pagos = self.pago_proveedor_repository
            .find_by_file_tour(file_tour_id)
            .await?;

        for pago in pagos {
            if !FINAL_STATUSES.contains(&pago.estado.as_str()) {
                self.pago_proveedor_repository
                    .update_status(pago.id, new_status)
                    .await?;
                result.pagos_proveedores_actualizados += 1;
                info!(
                    "PagoProveedor {} actualizado a '{}' por cascada de FileTour {}",
                    pago.id, new_status, file_tour_id
                );
            }
        }

        Ok(result)
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
            "FileTour {} actualizado de '{}' a '{}',",
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
                pagos_proveedores_actualizados: 0,
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
                pagos_proveedores_actualizados: 0,
            })
        }
    }
}
