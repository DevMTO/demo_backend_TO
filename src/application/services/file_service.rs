//! File Service - Lógica de negocio para archivos de viaje (Files)
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con files:
//! - Creación de files
//! - Actualización de files
//! - Búsqueda
//! - Logging de actividades
//! - Notificaciones
//! - Gestión de tours asociados (file_tours)
//! - **Confirmación de reservas con creación de pagos pendientes**

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::str::FromStr;
use chrono::{NaiveDate, Datelike, Duration, Utc};
use bigdecimal::BigDecimal;
use tracing::{info, warn, instrument};
use validator::Validate;

use crate::application::dtos::{
    CreateFileRequest, UpdateFileRequest, FileResponse, FileTourDto,
    ConfirmReservaRequest, ConfirmReservaResponse,
    CreateFileWithServicesRequest, CreateFileWithServicesResponse,
    UpdateFileWithServicesRequest, file_relations_dto::AssignEntradaToFileTourRequest,
};
use crate::application::ports::{FileRepositoryPort, PaginationOptions, NotificationServicePort};
use crate::application::ports::{FileTourRepositoryPort, FileTourInputData, PagoFileRepositoryPort, AgenciaRepositoryPort};
use crate::application::ports::{FileEntradaRepositoryPort, EntradaPrecioRepositoryPort, FileRestauranteRepositoryPort};
use crate::application::ports::{HotelRepositoryPort, UserRepositoryPort, PersonaRepositoryPort};
use crate::application::ports::{PagoProveedorRepositoryPort, FileVehiculoRepositoryPort, FileGuiaRepositoryPort, TarifaRepositoryPort};
use crate::application::services::{LoggingService, ContabilidadService};
use crate::application::services::file_status_service::FileStatusService;
use crate::domain::entities::{
    File, EntityType, UserRole,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::{NewPagoFileModel, UpdatePagoFileModel, FileEntradaModel, PagoFileModel};

const NON_EDITABLE_STATUSES: &[&str] = &["asignado", "en_curso", "completado"];

/// Servicio de files - contiene la lógica de negocio
pub struct FileService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
    pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    hotel_repository: Arc<dyn HotelRepositoryPort>,
    user_repository: Arc<dyn UserRepositoryPort>,
    file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
    file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    contabilidad_service: Arc<ContabilidadService>,
    persona_repository: Arc<dyn PersonaRepositoryPort>,
    file_status_service: Arc<FileStatusService>,
    pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
    file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    tarifa_repository: Arc<dyn TarifaRepositoryPort>,
}

impl FileService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        hotel_repository: Arc<dyn HotelRepositoryPort>,
        user_repository: Arc<dyn UserRepositoryPort>,
        file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
        entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
        file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
        contabilidad_service: Arc<ContabilidadService>,
        persona_repository: Arc<dyn PersonaRepositoryPort>,
        file_status_service: Arc<FileStatusService>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
        file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
        file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
        tarifa_repository: Arc<dyn TarifaRepositoryPort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
            logging_service,
            notification_service,
            pago_file_repository,
            agencia_repository,
            hotel_repository,
            user_repository,
            file_entrada_repository,
            entrada_precio_repository,
            file_restaurante_repository,
            contabilidad_service,
            persona_repository,
            file_status_service,
            pago_proveedor_repository,
            file_vehiculo_repository,
            file_guia_repository,
            tarifa_repository,
        }
    }

    /// Resolve user full name by user ID.
    /// Looks up the user's associated persona (nombre + apellidos).
    /// Falls back to username if no persona is linked.
    async fn get_user_full_name(&self, user_id: Option<i32>) -> Option<String> {
        let id = user_id?;
        let user = self.user_repository.find_by_id(id).await.ok().flatten()?;
        if let Some(persona_id) = user.id_persona {
            if let Ok(Some(persona)) = self.persona_repository.find_by_id(persona_id).await {
                return Some(persona.nombre_completo());
            }
        }
        Some(user.username)
    }

    /// Resolve user turno by user ID.
    async fn get_user_turno(&self, user_id: Option<i32>) -> Option<String> {
        let id = user_id?;
        let user = self.user_repository.find_by_id(id).await.ok().flatten()?;
        user.turno
    }

    /// Build a FileResponse with resolved user full names and turnos
    async fn build_file_response(&self, file: File, tours: Vec<FileTourDto>) -> FileResponse {
        let created_name = self.get_user_full_name(file.created_by).await;
        let updated_name = self.get_user_full_name(file.updated_by).await;
        let created_turno = self.get_user_turno(file.created_by).await;
        let updated_turno = self.get_user_turno(file.updated_by).await;
        FileResponse::from_file_with_tours(file, tours)
            .with_user_names(created_name, updated_name)
            .with_user_turnos(created_turno, updated_turno)
    }

    /// Carga los tours de un file con información completa del tour (INNER JOIN) y los convierte a DTO
    async fn load_file_tours(&self, file_id: i32) -> Result<Vec<FileTourDto>, ApplicationError> {
        let tours = self.file_tour_repository.find_by_file_with_tour(file_id).await?;
        Ok(tours.into_iter().map(|t| FileTourDto {
            id: t.id,
            id_tour: t.id_tour,
            orden: t.orden,
            precio_aplicado: t.precio_aplicado.clone(),
            notas: t.notas,
            nro_pasajeros: t.nro_pasajeros,
            fecha_tour: t.fecha_tour,
            turno_tour: t.turno_tour,
            lugar_recojo: t.lugar_recojo,
            hora_recojo: t.hora_recojo,
            geo_recojo: t.geo_recojo.and_then(|v| serde_json::from_value(v).ok()),
            status: t.status,
            tour_nombre: Some(t.tour_nombre),
            tour_lugar_inicio: t.tour_lugar_inicio,
            tour_lugar_fin: t.tour_lugar_fin,
            tour_precio_base: None,
            tour_duracion_dias: t.tour_duracion_dias,
            tour_tipo: t.tour_tipo,
            tour_is_active: Some(t.tour_is_active),
        }).collect())
    }

    /// Listar files con paginación
    #[instrument(skip(self))]
    pub async fn list_files(
        &self,
        options: PaginationOptions,
    ) -> Result<(Vec<FileResponse>, i64, i64), ApplicationError> {
        let result = self.file_repository
            .list_paginated(options)
            .await?;
        
        let total = result.total;
        let pages = result.pages();
        let current_page = result.current_page();
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in result.data {
            let tours = self.load_file_tours(file.id).await?;
            items.push(self.build_file_response(file, tours).await);
        }
        
        info!("Listados {} files (página {}, total: {})", items.len(), current_page, total);
        
        Ok((items, total, pages))
    }

    /// Obtener file por ID
    #[instrument(skip(self))]
    pub async fn get_file(&self, id: i32) -> Result<FileResponse, ApplicationError> {
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        let tours = self.load_file_tours(id).await?;
        let tours_len = tours.len();
        
        let response = self.build_file_response(file, tours).await;
        info!("File encontrado: ID {} con {} tours", id, tours_len);
        
        Ok(response)
    }

    /// Crear un nuevo file
    /// 
    /// Si el usuario tiene rol "Agencias", se usa su id_entidad como id_entidad automáticamente.
    /// Si el usuario es SuperAdmin o Admin, debe proporcionar id_entidad en el request.
    #[instrument(skip(self, request))]
    pub async fn create_file(
        &self,
        request: CreateFileRequest,
        created_by: i32,
        created_by_username: Option<String>,
        user_role: UserRole,
        user_id_entidad: Option<i32>,
    ) -> Result<FileResponse, ApplicationError> {
        // Resolver id_entidad según el rol del usuario
        let id_entidad_resolved = match user_role {
            UserRole::Agencias | UserRole::Hoteles | UserRole::HotelesGerente => {
                // Para agencias/hoteles, usar su id_entidad automáticamente
                user_id_entidad.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Usuario sin id_entidad configurado".to_string()
                    )
                })?
            },
            UserRole::HotelesGerenteCadena => {
                // Para gerente de cadena, debe proporcionar el id del hotel en el request
                // Validamos que el hotel proporcionado pertenezca a su cadena
                let target_hotel_id = request.id_entidad.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Debe seleccionar un hotel para crear el file".to_string()
                    )
                })?;

                let id_cadena = user_id_entidad.unwrap_or(0);
                if let Ok(Some(hotel)) = self.hotel_repository.find_by_id(target_hotel_id).await {
                    if hotel.id_cadena != id_cadena {
                        return Err(ApplicationError::Forbidden("El hotel seleccionado no pertenece a tu cadena".to_string()));
                    }
                } else {
                    return Err(ApplicationError::Validation("Hotel inválido".to_string()));
                }

                target_hotel_id
            },
            _ => {
                // Para superadmin/admin, debe venir en el request
                request.id_entidad.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Debe seleccionar una entidad para crear el file".to_string()
                    )
                })?
            }
        };

        // Obtener tours del request antes de consumirlo
        let tours_input = request.get_tours();
        if tours_input.is_empty() {
            return Err(ApplicationError::Validation(
                "Debe especificar al menos un tour para el file".to_string()
            ));
        }

        // Crear entidad de dominio con id_entidad resuelto
        let mut file = request.into_entity(Some(created_by), id_entidad_resolved);
        
        // Determinar tipo de entidad según el rol del usuario
        file.entidad = match user_role {
            UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena => Some("hoteles".to_string()),
            _ => Some("agencias".to_string()),
        };
        
        // Persistir el file
        let created = self.file_repository.create(&file).await?;
        info!("File creado: ID {} para fechas {} - {}", created.id, created.fecha_inicio, created.fecha_fin);
        
        // Insertar tours asociados (con fecha_tour y campos de recojo)
        let tours_data: Vec<FileTourInputData> = tours_input
            .into_iter()
            .enumerate()
            .map(|(idx, t)| {
                let orden = t.orden.unwrap_or((idx + 1) as i32);
                let precio = t.precio_aplicado.map(|p| BigDecimal::try_from(p).unwrap_or_default());
                // Convertir GeoLocation a JsonValue para la BD
                let geo_recojo_json = t.geo_recojo.and_then(|g| {
                    if g.has_data() {
                        serde_json::to_value(g).ok()
                    } else {
                        None
                    }
                });
                FileTourInputData {
                    id_tour: t.id_tour,
                    orden,
                    precio_aplicado: precio,
                    notas: t.notas.map(|n| serde_json::json!(n)),
                    fecha_tour: t.fecha_tour,
                    turno_tour: t.turno_tour,
                    lugar_recojo: t.lugar_recojo,
                    hora_recojo: t.hora_recojo,
                    status: t.status,
                    geo_recojo: geo_recojo_json,
                    nro_pasajeros: t.nro_pasajeros,
                }
            })
            .collect();
        
        let _created_tours = self.file_tour_repository
            .add_many(created.id, tours_data, Some(created_by))
            .await?;
        info!("{} tours asignados al file {}", _created_tours.len(), created.id);
        
        // Cargar tours con info completa (JOIN) para el response
        let tours_dto = self.load_file_tours(created.id).await?;
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<File>(
            Some(created_by),
            created_by_username.clone(),
            EntityType::File,
            created.id,
            &format!("File #{} - {} a {} ({} tours)", created.id, created.fecha_inicio, created.fecha_fin, tours_dto.len()),
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de file: {}", e);
        }
        
        // NOTE: Deliberadamente se omitido 'Notificación a admins'
        
        Ok(self.build_file_response(created, tours_dto).await)
    }

    /// Crear un file con tours, restaurantes, entradas y vehículos en una sola transacción atómica.
    /// Si cualquier operación falla, se hace rollback de todo.
    #[instrument(skip(self, request))]
    pub async fn create_file_with_services(
        &self,
        request: CreateFileWithServicesRequest,
        created_by: i32,
        created_by_username: Option<String>,
        user_role: UserRole,
        user_id_entidad: Option<i32>,
    ) -> Result<CreateFileWithServicesResponse, ApplicationError> {
        request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

        if request.tours.is_empty() {
            return Err(ApplicationError::Validation(
                "Debe especificar al menos un tour para el file".to_string()
            ));
        }

        let id_entidad_resolved = match user_role {
            UserRole::Agencias | UserRole::Hoteles | UserRole::HotelesGerente => {
                user_id_entidad.ok_or_else(|| {
                    ApplicationError::Validation("Usuario sin id_entidad configurado".to_string())
                })?
            },
            UserRole::HotelesGerenteCadena => {
                let target_hotel_id = request.id_entidad.ok_or_else(|| {
                    ApplicationError::Validation("Debe seleccionar un hotel para crear el file".to_string())
                })?;
                let id_cadena = user_id_entidad.unwrap_or(0);
                if let Ok(Some(hotel)) = self.hotel_repository.find_by_id(target_hotel_id).await {
                    if hotel.id_cadena != id_cadena {
                        return Err(ApplicationError::Forbidden("El hotel seleccionado no pertenece a tu cadena".to_string()));
                    }
                } else {
                    return Err(ApplicationError::Validation("Hotel inválido".to_string()));
                }
                target_hotel_id
            },
            _ => {
                request.id_entidad.ok_or_else(|| {
                    ApplicationError::Validation("Debe seleccionar una entidad para crear el file".to_string())
                })?
            }
        };

        let entidad = match user_role {
            UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena => Some("hoteles".to_string()),
            _ => Some("agencias".to_string()),
        };

        let fecha_inicio = NaiveDate::parse_from_str(&request.fecha_inicio, "%Y-%m-%d")
            .map_err(|_| ApplicationError::Validation("fecha_inicio inválida".to_string()))?;
        let fecha_fin = NaiveDate::parse_from_str(&request.fecha_fin, "%Y-%m-%d")
            .map_err(|_| ApplicationError::Validation("fecha_fin inválida".to_string()))?;

        let monto_total = BigDecimal::try_from(request.monto_total).unwrap_or_default();
        let nro_pasajeros = request.nro_pasajeros.unwrap_or(0);

        let response = self.file_tour_repository
            .create_file_with_services(
                id_entidad_resolved,
                entidad,
                fecha_inicio,
                fecha_fin,
                monto_total,
                nro_pasajeros,
                request.file_code,
                request.tours,
                Some(created_by),
            )
            .await?;

        info!("File {} creado con servicios (batch)", response.id);

        if let Err(e) = self.logging_service.log_create::<File>(
            Some(created_by),
            created_by_username,
            EntityType::File,
            response.id,
            &format!("File #{} - {} a {} ({} tours)", response.id, fecha_inicio, fecha_fin, response.tours.len()),
            None,
            None,
        ).await {
            warn!("Error al registrar log de creación de file con servicios: {}", e);
        }

        Ok(response)
    }

    /// Actualizar un file existente con tours, restaurantes, entradas y vehículos de forma inteligente.
    /// 
    /// Este método implementa lógica de diff para:
    /// - Tours removidos: cancela pagos y cascada el status
    /// - Tours modificados: maneja cambios en servicios anidados
    /// - Tours nuevos: crea con servicios asociados
    #[instrument(skip(self, request))]
    pub async fn update_file_with_services(
        &self,
        id: i32,
        request: UpdateFileWithServicesRequest,
        updated_by: i32,
    ) -> Result<CreateFileWithServicesResponse, ApplicationError> {
        request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let zero = BigDecimal::from(0);

        // Phase 1: Guard check - verify no tour has non-editable status
        for tour_input in &request.tours {
            if let Some(tour_id) = tour_input.id {
                if let Some(tour) = self.file_tour_repository.find_by_id(tour_id).await? {
                    if NON_EDITABLE_STATUSES.contains(&tour.status.as_str()) {
                        return Err(ApplicationError::Validation(
                            format!("Tour {} no puede ser modificado porque tiene status '{}'", tour_id, tour.status)
                        ));
                    }
                }
            }
        }

        // Phase 2: Load current state
        let existing_tours = self.file_tour_repository.find_by_file_with_tour(id).await?;
        let existing_pagos = self.pago_file_repository.find_all_by_file(id).await?;
        
        // Get file status to determine how to handle removed tours
        let file_status = self.file_repository.find_by_id(id).await?
            .map(|f| f.status)
            .unwrap_or_else(|| "reservado".to_string());
        
        let is_reservado = file_status == "reservado";

        let mut tour_restaurantes: HashMap<i32, Vec<_>> = HashMap::new();
        let mut tour_entradas: HashMap<i32, Vec<_>> = HashMap::new();
        let mut tour_vehiculos: HashMap<i32, Vec<_>> = HashMap::new();
        let mut tour_guias: HashMap<i32, Vec<_>> = HashMap::new();

        for ft in &existing_tours {
            tour_restaurantes.insert(ft.id, self.file_restaurante_repository.find_by_file_tour(ft.id).await?);
            tour_entradas.insert(ft.id, self.file_entrada_repository.find_by_file_tour(ft.id).await?);
            tour_vehiculos.insert(ft.id, self.file_vehiculo_repository.find_by_file_tour(ft.id).await?);
            tour_guias.insert(ft.id, self.file_guia_repository.find_by_file_tour_with_persona(ft.id).await?);
        }

        // Phase 3: Compute diff
        let request_tour_ids: HashSet<i32> = request.tours.iter().filter_map(|t| t.id).collect();

        let tours_to_cancel: Vec<_> = existing_tours.iter()
            .filter(|ft| !request_tour_ids.contains(&ft.id))
            .collect();

        let tours_to_update: Vec<_> = request.tours.iter()
            .filter(|t| t.id.is_some())
            .collect();

        let tours_to_create: Vec<_> = request.tours.iter()
            .filter(|t| t.id.is_none())
            .collect();

        // Phase 4: Handle removed tours (cancel or hard-delete based on file status)
        for ft in tours_to_cancel {
            if is_reservado {
                // For 'reservado' files: hard-delete services directly (no cancellation records)
                
                // Hard-delete entradas
                for fe in self.file_entrada_repository.find_by_file_tour(ft.id).await? {
                    self.file_entrada_repository.hard_delete(fe.id).await?;
                    info!("Entrada {} eliminada (hard delete) para tour {}", fe.id, ft.id);
                }
                
                // Hard-delete restaurantes
                for fr in self.file_restaurante_repository.find_by_file_tour(ft.id).await? {
                    self.file_restaurante_repository.hard_delete(fr.id).await?;
                    info!("Restaurante {} eliminado (hard delete) para tour {}", fr.id, ft.id);
                }
                
                // Hard-delete vehiculos
                for fv in self.file_vehiculo_repository.find_by_file_tour(ft.id).await? {
                    self.file_vehiculo_repository.hard_delete(fv.id).await?;
                    info!("Vehiculo {} eliminado (hard delete) para tour {}", fv.id, ft.id);
                }
                
                // Hard-delete guias
                for fg in self.file_guia_repository.find_by_file_tour_with_persona(ft.id).await? {
                    self.file_guia_repository.hard_delete(fg.id).await?;
                    info!("Guia {} eliminado (hard delete) para tour {}", fg.id, ft.id);
                }
                
                // Hard-delete file_tour
                self.file_tour_repository.hard_delete(ft.id).await?;
                info!("Tour {} eliminado del file (hard delete)", ft.id);
            } else {
                // For confirmed files: cancel services and create cancellation records
                let pagos_for_tour: Vec<_> = existing_pagos.iter()
                    .filter(|p| p.id_file_tour == Some(ft.id))
                    .collect();

                for pago in pagos_for_tour {
                    let mut update = UpdatePagoFileModel {
                        estado: Some("cancelado"),
                        tipo_registro: Some("cancelacion_tour"),
                        ..Default::default()
                    };

                    if &pago.monto_pagado > &zero {
                        update.monto_saldo_favor = Some(pago.monto_pagado.clone());
                        update.monto_pagado = Some(zero.clone());
                        update.saldo_autorizado = Some(true);
                        update.saldo_autorizado_por = Some(updated_by);
                        update.saldo_autorizado_at = Some(Utc::now());
                    }

                    self.pago_file_repository.update(pago.id, update).await?;
                    info!("Pago {} cancelado para tour {} eliminado", pago.id, ft.id);
                }

                // Cancel pagos_proveedores for this tour
                for pp in self.pago_proveedor_repository.find_by_file_tour(ft.id).await? {
                    self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                    info!("Pago proveedor {} cancelado por eliminación de tour {}", pp.id, ft.id);
                }

                // Cascade status to services
                self.file_status_service.update_file_tour_status(ft.id, "cancelado").await?;

                // Hard-delete file_tour
                self.file_tour_repository.hard_delete(ft.id).await?;
                info!("Tour {} cancelado y eliminado del file", ft.id);
            }
        }

        // Phase 5: Update remaining tours
        let mut new_entradas_to_update: Vec<(i32, i32, String)> = Vec::new();

        for tour_input in tours_to_update {
            let tour_id = tour_input.id.unwrap();
            let ft = existing_tours.iter().find(|f| f.id == tour_id).unwrap();

            // Get current services for this tour
            let current_rest = tour_restaurantes.get(&tour_id).and_then(|r| r.first()).cloned();
            let current_entradas = tour_entradas.get(&tour_id).cloned().unwrap_or_default();
            let current_vehiculos = tour_vehiculos.get(&tour_id).cloned().unwrap_or_default();
            let current_guias = tour_guias.get(&tour_id).cloned().unwrap_or_default();

            let pagos_for_tour: Vec<_> = existing_pagos.iter()
                .filter(|p| p.id_file_tour == Some(tour_id))
                .collect();

            // 5a. Handle Vehiculos (replace logic)
            if !current_vehiculos.is_empty() || tour_input.vehiculo.is_some() {
                // Remove old vehiculos
                for fv in &current_vehiculos {
                    if is_reservado {
                        // Hard-delete for reservado files
                        self.file_vehiculo_repository.hard_delete(fv.id).await?;
                        info!("Vehiculo {} eliminado (hard delete)", fv.id);
                    } else {
                        // Cancel for confirmed files
                        self.file_vehiculo_repository.update_status(fv.id, "cancelado").await?;
                        if let Some(pp) = self.pago_proveedor_repository
                            .find_by_file_relation("transporte", Some(fv.id), None, None, None).await?
                        {
                            self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                        }
                    }
                }
                // Create new if provided
                if let Some(ref veh) = tour_input.vehiculo {
                    let new_fv = self.file_vehiculo_repository.add(
                        tour_id, veh.id_vehiculo, veh.id_conductor,
                        veh.capacidad_asignada.unwrap_or(0), Some(updated_by)
                    ).await?;
                    // Child services start as 'asignado'
                    self.file_vehiculo_repository.update_status(new_fv.id, "asignado").await?;
                    // Create pago_proveedor
                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "transporte",
                        Some(veh.id_vehiculo),
                        None,
                        None,
                        None,
                        Some(tour_id),
                        Some(new_fv.id),
                        None,
                        None,
                        None,
                        None,
                        Some(updated_by),
                    ).await;
                }
            }

            // 5c. Handle Guias (replace logic)
            if !current_guias.is_empty() || tour_input.guia.is_some() {
                // Remove old guias
                for fg in &current_guias {
                    if is_reservado {
                        // Hard-delete for reservado files
                        self.file_guia_repository.hard_delete(fg.id).await?;
                        info!("Guia {} eliminado (hard delete)", fg.id);
                    } else {
                        // Cancel for confirmed files
                        self.file_guia_repository.update_status(fg.id, "cancelado").await?;
                        if let Some(pp) = self.pago_proveedor_repository
                            .find_by_file_relation("guia", None, None, Some(fg.id), None).await?
                        {
                            self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                        }
                    }
                }
                // Create new if provided
                if let Some(ref guia) = tour_input.guia {
                    let new_fg = self.file_guia_repository.add(
                        tour_id, guia.id_guia, guia.rol.as_deref(), Some(updated_by)
                    ).await?;
                    // Child services start as 'asignado'
                    self.file_guia_repository.update_status(new_fg.id, "asignado").await?;
                    // Create pago_proveedor
                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "guia",
                        None,
                        None,
                        Some(guia.id_guia),
                        None,
                        Some(tour_id),
                        None,
                        None,
                        Some(new_fg.id),
                        None,
                        None,
                        Some(updated_by),
                    ).await;
                }
            }

            // 5c. Handle Restaurantes
            match (&current_rest, &tour_input.restaurante) {
                (Some(old), None) => {
                    // Remove restaurant
                    if is_reservado {
                        // Hard-delete for reservado files
                        self.file_restaurante_repository.hard_delete(old.id).await?;
                        info!("Restaurante {} eliminado (hard delete)", old.id);
                    } else {
                        // Cancel for confirmed files
                        if let Some(pp) = self.pago_proveedor_repository
                            .find_by_file_relation("restaurante", None, Some(old.id), None, None).await?
                        {
                            self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                        }
                        self.file_restaurante_repository.update_status(old.id, "cancelado").await?;
                        
                        // Adjust monto_total in pagos_files only for confirmed files
                        if let Some(pago) = pagos_for_tour.iter().find(|p| p.tipo_registro == "deuda") {
                            if let Some(ref precio) = old.precio {
                                let new_monto = pago.monto_total.clone() - precio;
                                let update = UpdatePagoFileModel {
                                    monto_total: Some(new_monto),
                                    ..Default::default()
                                };
                                self.pago_file_repository.update(pago.id, update).await?;
                            }
                        }
                    }
                }
                (Some(old), Some(new)) if new.id_restaurante != old.id_restaurante => {
                    // Replace restaurant
                    if is_reservado {
                        // Hard-delete old for reservado files
                        self.file_restaurante_repository.hard_delete(old.id).await?;
                        info!("Restaurante {} eliminado (hard delete) por reemplazo", old.id);
                    } else {
                        // Cancel old for confirmed files
                        if let Some(pp) = self.pago_proveedor_repository
                            .find_by_file_relation("restaurante", None, Some(old.id), None, None).await?
                        {
                            self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                        }
                        self.file_restaurante_repository.update_status(old.id, "cancelado").await?;
                    }
                    
                    let new_fr = self.file_restaurante_repository.add(
                        tour_id, new.id_restaurante, new.tipo_servicio.as_deref(),
                        new.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by)
                    ).await?;
                    
                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "restaurante",
                        None,
                        Some(new.id_restaurante),
                        None,
                        None,
                        Some(tour_id),
                        None,
                        Some(new_fr.id),
                        None,
                        None,
                        new.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by),
                    ).await;

                    // Adjust monto_total only for confirmed files
                    if !is_reservado {
                        if let Some(pago) = pagos_for_tour.iter().find(|p| p.tipo_registro == "deuda") {
                            let old_precio_bd = old.precio.as_ref().unwrap_or(&zero);
                            let new_precio_bd = new.precio
                                .map(|p| BigDecimal::try_from(p).unwrap_or_default())
                                .unwrap_or_else(|| zero.clone());
                            let diff = &new_precio_bd - old_precio_bd;
                            if diff != zero {
                                let update = UpdatePagoFileModel {
                                    monto_total: Some(pago.monto_total.clone() + &diff),
                                    ..Default::default()
                                };
                                self.pago_file_repository.update(pago.id, update).await?;
                            }
                        }
                    }
                }
                (None, Some(new)) => {
                    // Add new
                    let new_fr = self.file_restaurante_repository.add(
                        tour_id, new.id_restaurante, new.tipo_servicio.as_deref(),
                        new.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by)
                    ).await?;
                    
                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "restaurante",
                        None,
                        Some(new.id_restaurante),
                        None,
                        None,
                        Some(tour_id),
                        None,
                        Some(new_fr.id),
                        None,
                        None,
                        new.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by),
                    ).await;

                    // Adjust monto_total
                    if let Some(pago) = pagos_for_tour.iter().find(|p| p.tipo_registro == "deuda") {
                        if let Some(precio) = new.precio {
                            let precio_bd = BigDecimal::try_from(precio).unwrap_or_default();
                            let update = UpdatePagoFileModel {
                                monto_total: Some(pago.monto_total.clone() + &precio_bd),
                                ..Default::default()
                            };
                            self.pago_file_repository.update(pago.id, update).await?;
                        }
                    }
                }
                _ => {}
            }

            // 5d. Handle Entradas (diff logic by composite key: id_entrada + id_entrada_precio)
            // Group current entradas by (id_entrada, id_entrada_precio)
            let current_by_key: HashMap<(i32, Option<i32>), &FileEntradaModel> = current_entradas.iter()
                .filter(|fe| fe.status != "cancelado")
                .map(|fe| ((fe.id_entrada, fe.id_entrada_precio), fe))
                .collect();

            // Group request entradas by (id_entrada, id_entrada_precio)
            let request_by_key: HashMap<(i32, Option<i32>), &AssignEntradaToFileTourRequest> = tour_input.entradas.as_ref()
                .map(|v| v.iter().map(|e| ((e.id_entrada, e.id_entrada_precio), e)).collect())
                .unwrap_or_default();

            // Entradas to add: in request but not in current (by composite key)
            let entradas_to_add: Vec<_> = request_by_key.iter()
                .filter(|(key, _)| !current_by_key.contains_key(key))
                .map(|(_, entrada)| *entrada)
                .collect();

            // Entradas to update: exists in both but cantidad changed
            let entradas_to_update: Vec<_> = request_by_key.iter()
                .filter(|(key, request_entrada)| {
                    if let Some(current_entrada) = current_by_key.get(key) {
                        current_entrada.cantidad != request_entrada.cantidad
                    } else {
                        false
                    }
                })
                .map(|(_, entrada)| *entrada)
                .collect();

            // Entradas to remove: in current but not in request (by composite key)
            // When id_entrada_precio changes for same id_entrada, the old combo is removed and new is added
            let entradas_to_remove: Vec<FileEntradaModel> = current_by_key.iter()
                .filter(|(key, _)| !request_by_key.contains_key(key))
                .map(|(_, fe)| (*fe).clone())
                .collect();

            // Remove entradas that are no longer in the request
            for fe in &entradas_to_remove {
                if is_reservado {
                    // Hard-delete for reservado files
                    self.file_entrada_repository.hard_delete(fe.id).await?;
                    info!("Entrada {} eliminada (hard delete)", fe.id);
                } else {
                    // Cancel for confirmed files
                    if let Some(pp) = self.pago_proveedor_repository
                        .find_by_file_relation("entrada", None, None, None, Some(fe.id)).await?
                    {
                        self.pago_proveedor_repository.update_status(pp.id, "cancelado").await?;
                    }
                    self.file_entrada_repository.update_status(fe.id, "cancelado").await?;
                }
            }

            // Add or update entradas using repository's upsert logic
            // The repository's add() method handles: add new OR update cantidad if same id_entrada + id_entrada_precio
            for entrada_input in &entradas_to_add {
                let new_fe = self.file_entrada_repository.add(
                    tour_id, entrada_input.id_entrada, entrada_input.cantidad,
                    entrada_input.id_entrada_precio, Some(updated_by)
                ).await?;

                // Track new entrada for later status update (reservado for reservado files, asignado for confirmed)
                let entrada_status = if is_reservado { "reservado" } else { "asignado" };
                new_entradas_to_update.push((new_fe.id, tour_id, entrada_status.to_string()));

                // Create pago_proveedor
                let monto = if let Some(precio_id) = entrada_input.id_entrada_precio {
                    if let Ok(Some(precio)) = self.entrada_precio_repository.find_by_id(precio_id).await {
                        Some(precio.precio * BigDecimal::from(entrada_input.cantidad))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let _ = self.contabilidad_service.auto_create_pago_proveedor(
                    "entrada",
                    None,
                    None,
                    None,
                    Some(entrada_input.id_entrada),
                    Some(tour_id),
                    None,
                    None,
                    None,
                    Some(new_fe.id),
                    monto,
                    Some(updated_by),
                ).await;
            }

            // Update existing entradas that have changed cantidad or id_entrada_precio
            for entrada_input in &entradas_to_update {
                // The add() method will find existing record and update cantidad
                self.file_entrada_repository.add(
                    tour_id, entrada_input.id_entrada, entrada_input.cantidad,
                    entrada_input.id_entrada_precio, Some(updated_by)
                ).await?;
            }

            // 5e. Update tour metadata and pagos_files
            // Formula: precio_aplicado = (base_tour_price × nro_pasajeros) + entradas_total + restaurante_price

            // Get file entity type for tariff lookup
            let tipo_entidad = match self.file_repository.find_by_id(id).await {
                Ok(Some(file)) => file.entidad.unwrap_or_else(|| "agencias".to_string()),
                _ => "agencias".to_string(),
            };

            // Get base price from tarifas table
            let base_price = self.tarifa_repository
                .find_by_tour_and_tipo(ft.id_tour, &tipo_entidad)
                .await?
                .map(|t| t.precio)
                .unwrap_or_else(|| ft.precio_aplicado.clone().unwrap_or_else(|| zero.clone()));

            // Get nro_pasajeros for this tour
            let nro_pasajeros = ft.nro_pasajeros.unwrap_or(0);

            // Re-fetch actual file_entradas from database to get correct state after add/remove
            let remaining_entradas = self.file_entrada_repository.find_by_file_tour(tour_id).await?;

            // Calculate entrada_precio from actual remaining file_entrada records
            let mut monto_entradas = zero.clone();
            for fe in &remaining_entradas {
                if fe.status != "cancelado" {
                    if let Some(precio_id) = fe.id_entrada_precio {
                        if let Ok(Some(precio)) = self.entrada_precio_repository.find_by_id(precio_id).await {
                            monto_entradas += precio.precio * BigDecimal::from(fe.cantidad);
                        }
                    }
                }
            }

            // Calculate restaurante price
            let mut restaurante_price = zero.clone();
            if let Some(ref rest) = current_rest {
                if rest.status != "cancelado" {
                    if let Some(ref precio) = rest.precio {
                        restaurante_price = precio.clone();
                    }
                }
            }

            // Calculate precio_aplicado: (base_price × nro_pasajeros) + entradas + restaurante
            let base_tour_total = &base_price * BigDecimal::from(nro_pasajeros);
            let new_precio = &base_tour_total + &monto_entradas + &restaurante_price;

            let entrada_precio = if monto_entradas > zero { Some(monto_entradas) } else { None };

            // Update ALL pagos_files records for this tour (deuda, pago, pago_final)
            // Update monto_total on all records
            for pago in pagos_for_tour.iter() {
                if pago.monto_total != new_precio {
                    let update = UpdatePagoFileModel {
                        monto_total: Some(new_precio.clone()),
                        ..Default::default()
                    };
                    self.pago_file_repository.update(pago.id, update).await?;
                }
            }

            // Only update entrada_precio on the deuda record (not on pago/pago_final)
            if let Some(deuda_pago) = pagos_for_tour.iter().find(|p| p.tipo_registro == "deuda") {
                if deuda_pago.entrada_precio != entrada_precio {
                    let update = UpdatePagoFileModel {
                        entrada_precio: Some(entrada_precio.clone()),
                        ..Default::default()
                    };
                    self.pago_file_repository.update(deuda_pago.id, update).await?;
                }
            }

            // Update file_tour record
            if let Some(existing_ft) = self.file_tour_repository.find_by_id(tour_id).await? {
                let orden = tour_input.orden.unwrap_or(ft.orden);
                let status = tour_input.status.clone().unwrap_or_else(|| ft.status.clone());
                
                let updated_ft = crate::infrastructure::persistence::models::FileTourModel {
                    orden,
                    precio_aplicado: Some(new_precio.clone()),
                    fecha_tour: tour_input.fecha_tour.as_ref().and_then(|s| {
                        let s = s.trim();
                        if s.is_empty() { None } else { NaiveDate::parse_from_str(s, "%Y-%m-%d").ok() }
                    }).or(ft.fecha_tour),
                    turno_tour: tour_input.turno_tour.clone().or_else(|| ft.turno_tour.clone()),
                    lugar_recojo: tour_input.lugar_recojo.clone().or_else(|| ft.lugar_recojo.clone()),
                    hora_recojo: tour_input.hora_recojo.as_ref().and_then(|s| {
                        let s = s.trim();
                        if s.is_empty() { None } else if s.len() == 5 {
                            chrono::NaiveTime::parse_from_str(s, "%H:%M").ok()
                        } else {
                            chrono::NaiveTime::parse_from_str(s, "%H:%M:%S").ok()
                        }
                    }).or(ft.hora_recojo),
                    status,
                    nro_pasajeros: tour_input.nro_pasajeros.or(ft.nro_pasajeros),
                    ..existing_ft
                };
                self.file_tour_repository.update(&updated_ft).await?;

                // Redistribute excess payment if sum(monto_pagado) > new_monto_total
                self.redistribute_excess_payment(id, tour_id, &new_precio, updated_by).await?;
            }
        }

        // Phase 5b: Update new entrada statuses (reservado for reservado files, asignado for confirmed)
        let new_entrada_status = if is_reservado { "reservado" } else { "asignado" };
        for (entrada_id, _, _) in new_entradas_to_update {
            self.file_entrada_repository.update_status(entrada_id, new_entrada_status).await?;
        }

        // Phase 6: Create new tours
        for tour_input in tours_to_create {
            let orden = tour_input.orden.unwrap_or(1);

            // Use precio_aplicado from request (following confirm_reserva pattern)
            // precio_aplicado is the contractual/tariff price, set when tour is priced
            let precio_aplicado = tour_input.precio_aplicado
                .map(|p| BigDecimal::try_from(p).unwrap_or_default())
                .unwrap_or_else(|| zero.clone());

            let hora_recojo = tour_input.hora_recojo.as_ref().and_then(|s| {
                let s = s.trim();
                if s.is_empty() { None } else if s.len() == 5 {
                    chrono::NaiveTime::parse_from_str(s, "%H:%M").ok()
                } else {
                    chrono::NaiveTime::parse_from_str(s, "%H:%M:%S").ok()
                }
            });

            let fecha_tour = tour_input.fecha_tour.as_ref().and_then(|s| {
                let s = s.trim();
                if s.is_empty() { None } else { NaiveDate::parse_from_str(s, "%Y-%m-%d").ok() }
            });

            // Store status for later use when creating child services
            let tour_status = tour_input.status.clone().unwrap_or_else(|| "reservado".to_string());

            // Create tour using repository's add_many method
            let created_tours = self.file_tour_repository.add_many(id, vec![FileTourInputData {
                id_tour: tour_input.id_tour,
                orden,
                precio_aplicado: Some(precio_aplicado.clone()),
                notas: None,
                fecha_tour,
                turno_tour: tour_input.turno_tour.clone(),
                lugar_recojo: tour_input.lugar_recojo.clone(),
                hora_recojo,
                status: Some(tour_status.clone()),
                geo_recojo: None,
                nro_pasajeros: tour_input.nro_pasajeros,
            }], Some(updated_by)).await?;

            if let Some(new_ft) = created_tours.first() {
                let ft_id = new_ft.id;

                // Create restaurante
                if let Some(ref rest) = tour_input.restaurante {
                    let new_fr = self.file_restaurante_repository.add(
                        ft_id, rest.id_restaurante, rest.tipo_servicio.as_deref(),
                        rest.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by)
                    ).await?;

                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "restaurante",
                        None,
                        Some(rest.id_restaurante),
                        None,
                        None,
                        Some(ft_id),
                        None,
                        Some(new_fr.id),
                        None,
                        None,
                        rest.precio.map(|p| BigDecimal::try_from(p).unwrap_or_default()),
                        Some(updated_by),
                    ).await;
                }

                // Create entradas
                if let Some(entradas) = &tour_input.entradas {
                    for entrada in entradas {
                        let new_fe = self.file_entrada_repository.add(
                            ft_id, entrada.id_entrada, entrada.cantidad,
                            entrada.id_entrada_precio, Some(updated_by)
                        ).await?;

                        // Child services: reservado for reservado files, asignado for confirmed
                        let child_status = if is_reservado { "reservado" } else { "asignado" };
                        self.file_entrada_repository.update_status(new_fe.id, child_status).await?;

                        let monto = if let Some(precio_id) = entrada.id_entrada_precio {
                            if let Ok(Some(precio)) = self.entrada_precio_repository.find_by_id(precio_id).await {
                                Some(precio.precio * BigDecimal::from(entrada.cantidad))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let _ = self.contabilidad_service.auto_create_pago_proveedor(
                            "entrada",
                            None,
                            None,
                            None,
                            Some(entrada.id_entrada),
                            Some(ft_id),
                            None,
                            None,
                            None,
                            Some(new_fe.id),
                            monto,
                            Some(updated_by),
                        ).await;
                    }
                }

                // Create vehiculo
                if let Some(ref veh) = tour_input.vehiculo {
                    let new_fv = self.file_vehiculo_repository.add(
                        ft_id, veh.id_vehiculo, veh.id_conductor,
                        veh.capacidad_asignada.unwrap_or(0), Some(updated_by)
                    ).await?;

                    // Child services: reservado for reservado files, asignado for confirmed
                    let child_status = if is_reservado { "reservado" } else { "asignado" };
                    self.file_vehiculo_repository.update_status(new_fv.id, child_status).await?;

                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "transporte",
                        Some(veh.id_vehiculo),
                        None,
                        None,
                        None,
                        Some(ft_id),
                        Some(new_fv.id),
                        None,
                        None,
                        None,
                        None,
                        Some(updated_by),
                    ).await;
                }

                // Create guia
                if let Some(ref guia) = tour_input.guia {
                    let new_fg = self.file_guia_repository.add(
                        ft_id, guia.id_guia, guia.rol.as_deref(), Some(updated_by)
                    ).await?;

                    // Child services: reservado for reservado files, asignado for confirmed
                    let child_status = if is_reservado { "reservado" } else { "asignado" };
                    self.file_guia_repository.update_status(new_fg.id, child_status).await?;

                    let _ = self.contabilidad_service.auto_create_pago_proveedor(
                        "guia",
                        None,
                        None,
                        Some(guia.id_guia),
                        None,
                        Some(ft_id),
                        None,
                        None,
                        Some(new_fg.id),
                        None,
                        None,
                        Some(updated_by),
                    ).await;
                }
            }
        }

        // Phase 7: Update file metadata
        if let Some(fecha_inicio) = &request.fecha_inicio {
            if let Ok(date) = NaiveDate::parse_from_str(fecha_inicio, "%Y-%m-%d") {
                if let Ok(Some(mut file)) = self.file_repository.find_by_id(id).await {
                    file.fecha_inicio = date;
                    file.updated_by = Some(updated_by);
                    self.file_repository.update(&file).await.ok();
                }
            }
        }
        if let Some(fecha_fin) = &request.fecha_fin {
            if let Ok(date) = NaiveDate::parse_from_str(fecha_fin, "%Y-%m-%d") {
                if let Ok(Some(mut file)) = self.file_repository.find_by_id(id).await {
                    file.fecha_fin = date;
                    file.updated_by = Some(updated_by);
                    self.file_repository.update(&file).await.ok();
                }
            }
        }
        if let Some(nro_pasajeros) = request.nro_pasajeros {
            if let Ok(Some(mut file)) = self.file_repository.find_by_id(id).await {
                file.nro_pasajeros = nro_pasajeros;
                file.updated_by = Some(updated_by);
                self.file_repository.update(&file).await.ok();
            }
        }
        if let Some(status) = &request.status {
            self.file_repository.update_status(id, status).await.ok();
        }

        // Phase 8: Recalculate file monto_total only (not monto_pagado)
        self.recalculate_file_monto_total(id).await?;

        // Get final response
        let final_tours = self.file_tour_repository.find_by_file_with_tour(id).await?;
        let file = self.file_repository.find_by_id(id).await?.unwrap();

        let tours_response: Vec<_> = final_tours.iter()
            .map(|t| crate::application::dtos::file_dto::FileTourBasicDto {
                id: t.id,
                id_tour: t.id_tour,
                orden: t.orden,
                status: t.status.clone(),
            })
            .collect();

        info!("File {} actualizado con servicios (diff logic)", id);

        if let Err(e) = self.logging_service.log_update::<File>(
            Some(updated_by),
            None,
            EntityType::File,
            id,
            None,
            None,
            None,
            None,
        ).await {
            warn!("Error al registrar log de actualización de file con servicios: {}", e);
        }

        Ok(CreateFileWithServicesResponse {
            id: file.id,
            file_code: file.file_code,
            status: file.status,
            tours: tours_response,
        })
    }

    /// Actualizar un file existente
    #[instrument(skip(self, request))]
    pub async fn update_file(
        &self,
        id: i32,
        request: UpdateFileRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<FileResponse, ApplicationError> {
        // Verificar que existe
        let old_file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Detectar campos cambiados
        let mut changed_fields = self.detect_changed_fields(&old_file, &request);
        
        // Obtener tours del request antes de consumirlo
        let tours_update = request.get_tours();
        
        // Aplicar cambios
        let updated_entity = request.apply_to(old_file.clone(), Some(updated_by));
        
        // Persistir file
        let result = self.file_repository.update(&updated_entity).await?;
        info!("✏️ File actualizado: ID {}", result.id);
        
        // Actualizar tours si se especificaron
        let tours_dto = if let Some(tours_input) = tours_update {
            // Eliminar todos los tours existentes
            self.file_tour_repository.remove_by_file(id).await?;
            
            // Insertar nuevos tours (con fecha_tour y campos de recojo)
            let tours_data: Vec<FileTourInputData> = tours_input
                .into_iter()
                .enumerate()
                .map(|(idx, t)| {
                    let orden = t.orden.unwrap_or((idx + 1) as i32);
                    let precio = t.precio_aplicado.map(|p| BigDecimal::try_from(p).unwrap_or_default());
                    // Convertir GeoLocation a JsonValue para la BD
                    let geo_recojo_json = t.geo_recojo.and_then(|g| {
                        if g.has_data() {
                            serde_json::to_value(g).ok()
                        } else {
                            None
                        }
                    });
                    FileTourInputData {
                        id_tour: t.id_tour,
                        orden,
                        precio_aplicado: precio,
                        notas: t.notas.map(|n| serde_json::json!(n)),
                        fecha_tour: t.fecha_tour,
                        turno_tour: t.turno_tour,
                        lugar_recojo: t.lugar_recojo,
                        hora_recojo: t.hora_recojo,
                        status: t.status,
                        geo_recojo: geo_recojo_json,
                        nro_pasajeros: t.nro_pasajeros,
                    }
                })
                .collect();
            
            let created_tours = self.file_tour_repository
                .add_many(id, tours_data, Some(updated_by))
                .await?;
            
            changed_fields.push("tours".to_string());
            info!("Tours actualizados para file {}: {} tours", id, created_tours.len());
            
            // Cargar con JOIN para info completa
            self.load_file_tours(id).await?
        } else {
            // Cargar tours existentes
            self.load_file_tours(id).await?
        };
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(updated_by),
            updated_by_username.clone(),
            EntityType::File,
            id,
            Some(&old_file),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de file: {}", e);
        }
        
        // Notificación si hubo cambios importantes (especialmente estado)
        if !changed_fields.is_empty() {
            let username = updated_by_username.unwrap_or_else(|| "Sistema".to_string());
            let priority = if changed_fields.contains(&"status".to_string()) {
                NotificationPriority::High
            } else {
                NotificationPriority::Normal
            };
            
            if let Err(e) = self.notification_service.notify_roles(
                vec![UserRole::SuperAdmin, UserRole::Admin],
                "File actualizado",
                &format!("{} ha actualizado file #{} (campos: {})", username, id, changed_fields.join(", ")),
                NotificationType::Info,
                NotificationCategory::Crud,
                priority,
                Some(updated_by),
            ).await {
                warn!("Error al enviar notificación de file actualizado: {}", e);
            }
        }
        
        Ok(self.build_file_response(result, tours_dto).await)
    }

    /// Desactivar un file (soft delete)
    #[instrument(skip(self))]
    pub async fn delete_file(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener file antes de desactivar
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Soft delete
        let deleted = self.file_repository.soft_delete(id, deleted_by).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        info!("[DELETE] File desactivado: ID {}", id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<File>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::File,
            id,
            Some(&file),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de file: {}", e);
        }
        
        // Notificación a admins
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File desactivado",
            &format!("{} ha desactivado el file #{} (fechas: {} - {})", username, id, file.fecha_inicio, file.fecha_fin),
            NotificationType::Warning,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de file desactivado: {}", e);
        }
        
        Ok(())
    }

    /// Restaurar un file desactivado
    #[instrument(skip(self))]
    pub async fn restore_file(
        &self,
        id: i32,
        restored_by: i32,
        restored_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Restore via repository
        if !self.file_repository.restore(id, restored_by).await? {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        // Obtener file restaurado
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        info!("♻️ File restaurado: ID {}", id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(restored_by),
            restored_by_username.clone(),
            EntityType::File,
            id,
            None,
            Some(&file),
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de restauración de file: {}", e);
        }
        
        // Notificación a admins - Success
        let username = restored_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File restaurado",
            &format!("{} ha restaurado el file #{} (fechas: {} - {})", username, id, file.fecha_inicio, file.fecha_fin),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::Normal,
            Some(restored_by),
        ).await {
            warn!("Error al enviar notificación de file restaurado: {}", e);
        }
        
        Ok(())
    }

    /// Eliminación permanente de file (hard delete) - Solo SuperAdmin
    #[instrument(skip(self))]
    pub async fn hard_delete_file(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<(), ApplicationError> {
        // Obtener file antes de eliminar para el log
        let file = self.file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id)))?;
        
        // Eliminar permanentemente
        let deleted = self.file_repository.hard_delete(id).await?;
        
        if !deleted {
            return Err(ApplicationError::NotFound(format!("File {} no encontrado", id)));
        }
        
        info!("🗑️ File ELIMINADO PERMANENTEMENTE: ID {}", id);
        
        // Logging del evento (acción crítica)
        if let Err(e) = self.logging_service.log_delete::<File>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::File,
            id,
            Some(&file),
            Some("HARD_DELETE - Eliminación permanente".to_string()),
        ).await {
            warn!("Error al registrar log de eliminación permanente de file: {}", e);
        }
        
        // Notificación CRÍTICA a SuperAdmin únicamente
        let username = deleted_by_username.unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin],
            "⚠️ FILE ELIMINADO PERMANENTEMENTE",
            &format!(
                "ACCIÓN CRÍTICA: {} ha eliminado PERMANENTEMENTE el file #{} (fechas: {} - {}). Esta acción NO se puede deshacer.",
                username, id, file.fecha_inicio, file.fecha_fin
            ),
            NotificationType::Error,
            NotificationCategory::Alert,
            NotificationPriority::Urgent,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de eliminación permanente de file: {}", e);
        }
        
        Ok(())
    }

    /// Listar files por agencia
    #[instrument(skip(self))]
    pub async fn list_files_by_agencia(&self, agencia_id: i32, entidad: Option<&str>) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_by_entidad(agencia_id, entidad)
            .await?;
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in files {
            let tours = self.load_file_tours(file.id).await?;
            items.push(self.build_file_response(file, tours).await);
        }
        
        info!("{} files encontrados para entidad {} {:?} (con tours cargados)", items.len(), agencia_id, entidad);
        Ok(items)
    }

    /// Obtener file_codes de files activos (no completado/cancelado/no_show/anulado) por entidad
    pub async fn get_active_file_codes(&self, id_entidad: i32, entidad: Option<&str>) -> Result<Vec<String>, ApplicationError> {
        self.file_repository.find_active_file_codes(id_entidad, entidad).await
    }

    /// Buscar files por rango de fechas
    #[instrument(skip(self))]
    pub async fn search_files_by_date_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_by_date_range(from, to)
            .await?;
        
        // Cargar tours para cada file
        let mut items = Vec::new();
        for file in files {
            let tours = self.load_file_tours(file.id).await?;
            items.push(self.build_file_response(file, tours).await);
        }
        
        info!("{} files encontrados entre {} y {} (con tours cargados)", items.len(), from, to);
        Ok(items)
    }

    /// Listar files próximos (en los próximos 7 días)
    #[instrument(skip(self))]
    pub async fn list_files_upcoming(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_upcoming()
            .await?;
        
        info!("{} files próximos encontrados", files.len());
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Listar files con pago pendiente
    #[instrument(skip(self))]
    pub async fn list_files_pending_payment(&self) -> Result<Vec<FileResponse>, ApplicationError> {
        let files = self.file_repository
            .find_pending_payment()
            .await?;
        
        info!("{} files con pago pendiente encontrados", files.len());
        Ok(files.into_iter().map(Into::into).collect())
    }

    /// Detectar campos que cambiaron
    fn detect_changed_fields(&self, old: &File, request: &UpdateFileRequest) -> Vec<String> {
        let mut changed = Vec::new();
        
        if request.fecha_inicio.as_ref().map(|f| *f != old.fecha_inicio).unwrap_or(false) {
            changed.push("fecha_inicio".to_string());
        }
        if request.fecha_fin.as_ref().map(|f| *f != old.fecha_fin).unwrap_or(false) {
            changed.push("fecha_fin".to_string());
        }
        // id_tour eliminado - tours ahora están en file_tours
        if request.tours.is_some() {
            changed.push("tours".to_string());
        }
        if request.id_entidad.as_ref().map(|a| *a != old.id_entidad).unwrap_or(false) {
            changed.push("id_entidad".to_string());
        }
        if request.status.as_ref().map(|s| s != &old.status).unwrap_or(false) {
            changed.push("status".to_string());
        }
        if request.nro_pasajeros.as_ref().map(|n| *n != old.nro_pasajeros).unwrap_or(false) {
            changed.push("nro_pasajeros".to_string());
        }
        
        changed
    }

    // =========================================================================
    // CONFIRMACIÓN DE RESERVA
    // =========================================================================

    /// Confirmar una reserva (file)
    /// 
    /// Este método:
    /// 1. Verifica que el file exista y esté en estado "reservado"
    /// 2. Actualiza el status a "confirmado"
    /// 3. Crea un registro en pagos_files con estado "pendiente"
    /// 4. Notifica a los admins
    /// 5. Registra en el log de actividad
    #[instrument(skip(self))]
    pub async fn confirmar_reserva(
        &self,
        request: ConfirmReservaRequest,
        confirmed_by: i32,
        confirmed_by_username: Option<String>,
    ) -> Result<ConfirmReservaResponse, ApplicationError> {
        // 1. Obtener el file
        let file = self.file_repository
            .find_by_id(request.file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(
                format!("File {} no encontrado", request.file_id)
            ))?;
        
        // 2. Verificar que el status sea válido para confirmar
        let valid_statuses = ["pendiente", "reservado"];
        if !valid_statuses.contains(&file.status.as_str()) {
            return Err(ApplicationError::Validation(format!(
                "El file no puede ser confirmado. Estado actual: '{}'. Estados válidos para confirmación: pendiente, reservado",
                file.status
            )));
        }
        
        // Verificar que no existan ya pagos_file para este file
        let existing_pagos = self.pago_file_repository.find_all_by_file(request.file_id).await?;
        if !existing_pagos.is_empty() {
            return Err(ApplicationError::Validation(
                "Este file ya tiene registros de pago asociados".to_string()
            ));
        }
        
        // 3. Obtener política de pago según tipo de entidad
        let (pago_anticipado, tipo_vencimiento) = if file.entidad.as_deref() == Some("hoteles") {
            // Hoteles: sin pago anticipado, vencimiento mensual por defecto
            (false, Some("mensual".to_string()))
        } else {
            // Agencias: usar política de pago configurada
            let agencia = self.agencia_repository
                .find_by_id(file.id_entidad)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(
                    format!("Agencia {} no encontrada", file.id_entidad)
                ))?;
            (agencia.pago_anticipado, agencia.tipo_vencimiento)
        };
        
        // 4. Calcular montos y fechas
        let monto_total = request.monto_total
            .map(|m| BigDecimal::try_from(m).unwrap_or_default())
            .unwrap_or_else(|| file.monto_total.clone());
        
        // Calcular fecha_vencimiento para pago anticipado (igual para todos los tours)
        let fecha_vencimiento_anticipado = if pago_anticipado {
            let tours_preview = self.file_tour_repository.find_by_file_with_tour(request.file_id).await?;
            let earliest_tour_date = tours_preview.iter()
                .filter_map(|t| t.fecha_tour)
                .min();

            Some(match earliest_tour_date {
                Some(fecha) => fecha - chrono::Duration::days(1),
                None => {
                    warn!("⚠️ Entidad con pago_anticipado pero file {} sin tours con fecha", request.file_id);
                    (Utc::now() + Duration::days(7)).date_naive()
                }
            })
        } else {
            None
        };
        
        // 5. Actualizar el file a status "confirmado"
        let mut updated_file = file.clone();
        updated_file.status = "confirmado".to_string();
        updated_file.monto_total = monto_total.clone();
        updated_file.updated_by = Some(confirmed_by);
        updated_file.updated_at = Utc::now();
        
        let updated_file = self.file_repository.update(&updated_file).await?;
        info!("✅ File {} confirmado", request.file_id);
        
        // 6. Crear un pago_file (deuda) POR CADA file_tour del file
        let tours = self.file_tour_repository.find_by_file_with_tour(request.file_id).await?;
        
        if tours.is_empty() {
            return Err(ApplicationError::Validation(
                "El file no tiene tours asociados. No se puede confirmar sin file_tours.".to_string()
            ));
        }
        
        let mut pago_file_ids: Vec<i32> = Vec::new();
        let mut primera_fecha_vencimiento: Option<NaiveDate> = fecha_vencimiento_anticipado;

        for ft in &tours {
            // Calcular monto del tour: precio_aplicado (debe estar seteado desde la tarifa)
            let monto_tour = ft.precio_aplicado.clone()
                .unwrap_or_else(|| BigDecimal::from(0));

            // Calcular monto de entradas para este file_tour
            let entradas_ft = self.file_entrada_repository.find_by_file_tour(ft.id).await.unwrap_or_default();
            let zero = BigDecimal::from(0);
            let mut monto_entradas = zero.clone();
            let tiene_entradas = !entradas_ft.is_empty();

            for fe in &entradas_ft {
                if let Some(precio_id) = fe.id_entrada_precio {
                    if let Ok(Some(precio)) = self.entrada_precio_repository.find_by_id(precio_id).await {
                        monto_entradas += &precio.precio * BigDecimal::from(fe.cantidad);
                    }
                }
            }

            // Calcular monto de restaurantes para este file_tour
            let restaurantes_ft = self.file_restaurante_repository.find_by_file_tour(ft.id).await.unwrap_or_default();
            let _tiene_restaurantes = !restaurantes_ft.is_empty();
            let mut monto_restaurantes = zero.clone();

            for fr in &restaurantes_ft {
                if let Some(precio) = &fr.precio {
                    monto_restaurantes += precio;
                }
            }

            // Calcular fecha_vencimiento según política de pago
            let fecha_vencimiento = if let Some(fv) = fecha_vencimiento_anticipado {
                // Pago anticipado: misma fecha para todos los tours
                fv
            } else {
                // Calcular según tipo_vencimiento y fecha del tour
                let fecha_tour = ft.fecha_tour.unwrap_or_else(|| Utc::now().date_naive());
                calcular_fecha_vencimiento(
                    tipo_vencimiento.as_deref().unwrap_or("mensual"),
                    fecha_tour,
                )
            };

            if primera_fecha_vencimiento.is_none() {
                primera_fecha_vencimiento = Some(fecha_vencimiento);
            }

            let new_pago = NewPagoFileModel {
                id_file: request.file_id,
                id_entidad: file.id_entidad,
                entidad: file.entidad.as_deref(),
                monto_total: monto_tour,
                monto_pagado: zero.clone(),
                estado: "pendiente",
                fecha_vencimiento: Some(fecha_vencimiento),
                notas: request.notas.as_deref(),
                created_by: Some(confirmed_by),
                id_file_tour: Some(ft.id),
                tipo_registro: "deuda",
                monto_saldo_favor: None,
                saldo_autorizado: false,
                saldo_autorizado_por: None,
                saldo_autorizado_at: None,
                entradas: tiene_entradas,
                entrada_precio: if tiene_entradas { Some(monto_entradas) } else { None },
                cuota: Some(0),
                pagado_por: None,
                pagado_at: None,
            };

            let pago = self.pago_file_repository.create(new_pago).await?;
            info!("💰 Deuda creada: pago_file ID {} para file_tour {} (file {}, entradas: {})", pago.id, ft.id, request.file_id, tiene_entradas);
            pago_file_ids.push(pago.id);

            // AUTO-CREAR PAGOS PROVEEDOR (entradas)
            for fe in &entradas_ft {
                let monto = if let Some(precio_id) = fe.id_entrada_precio {
                    match self.entrada_precio_repository.find_by_id(precio_id).await {
                        Ok(Some(precio)) => Some(precio.precio * BigDecimal::from(fe.cantidad)),
                        _ => None,
                    }
                } else {
                    None
                };

                if let Err(e) = self.contabilidad_service
                    .auto_create_pago_proveedor(
                        "entrada",
                        None,
                        None,
                        None,
                        Some(fe.id_entrada),
                        Some(ft.id),
                        None,
                        None,
                        None,
                        Some(fe.id),
                        monto,
                        Some(confirmed_by),
                    ).await
                {
                    warn!("Error al auto-crear pago proveedor para entrada {}: {}", fe.id_entrada, e);
                }
            }

            // AUTO-CREAR PAGOS PROVEEDOR (restaurantes)
            for fr in &restaurantes_ft {
                if let Err(e) = self.contabilidad_service
                    .auto_create_pago_proveedor(
                        "restaurante",
                        None,
                        Some(fr.id_restaurante),
                        None,
                        None,
                        Some(ft.id),
                        None,
                        Some(fr.id),
                        None,
                        None,
                        fr.precio.clone(),
                        Some(confirmed_by),
                    ).await
                {
                    warn!("Error al auto-crear pago proveedor para restaurante {}: {}", fr.id_restaurante, e);
                }
            }
        }

        let fecha_vencimiento_display = primera_fecha_vencimiento
            .unwrap_or_else(|| Utc::now().date_naive());
        
        info!("💰 {} deudas creadas para file {}", pago_file_ids.len(), request.file_id);
        
        // 7. Registrar en el log de actividad
        let username = confirmed_by_username.clone().unwrap_or_else(|| "Sistema".to_string());
        if let Err(e) = self.logging_service.log_update::<File>(
            Some(confirmed_by),
            confirmed_by_username.clone(),
            EntityType::File,
            request.file_id,
            Some(&file),
            Some(&updated_file),
            Some(vec!["status".to_string(), "confirmacion".to_string()]),
            None, // IP no aplica en operación de servicio
        ).await {
            warn!("Error al registrar log de confirmación: {}", e);
        }
        
        // 8. Notificar a los admins
        let entity_nombre = if file.entidad.as_deref() == Some("hoteles") {
            self.hotel_repository.find_by_id(file.id_entidad).await.ok().flatten().map(|h| h.nombre)
        } else {
            self.agencia_repository.find_by_id(file.id_entidad).await.ok().flatten().map(|a| a.nombre)
        }.unwrap_or_else(|| format!("Entidad #{}", file.id_entidad));

        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "📋 Reserva Confirmada",
            &format!(
                "{} ha confirmado la reserva #{} (File #{}).\nEntidad: {}\nMonto total: S/ {}\nVencimiento de pago: {}",
                username,
                updated_file.file_code.clone().unwrap_or_else(|| format!("F-{}", request.file_id)),
                request.file_id,
                entity_nombre,
                monto_total,
                fecha_vencimiento_display
            ),
            NotificationType::Success,
            NotificationCategory::Crud,
            NotificationPriority::High,
            Some(confirmed_by),
        ).await {
            warn!("Error al enviar notificación de confirmación: {}", e);
        }
        
        // 9. Notificar también al contador/gerente de la entidad específica (filtrado por id_entidad)
        if let Err(e) = self.notification_service.notify_roles_for_entity(
            vec![UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::HotelesGerente, UserRole::HotelesGerenteCadena],
            file.id_entidad,
            "💰 Nuevo pago pendiente",
            &format!(
                "Se ha confirmado la reserva #{} con un monto de S/ {}.\nFecha de vencimiento: {}\nPor favor, gestione el pago.",
                updated_file.file_code.clone().unwrap_or_else(|| format!("F-{}", request.file_id)),
                monto_total,
                fecha_vencimiento_display
            ),
            NotificationType::Warning,
            NotificationCategory::Financial,
            NotificationPriority::High,
            Some(confirmed_by),
        ).await {
            warn!("Error al enviar notificación al contador de la agencia: {}", e);
        }
        
        // 10. Cargar tours para el response
        let tours_dto = self.load_file_tours(request.file_id).await?;
        
        let num_deudas = pago_file_ids.len();
        let mensaje = format!(
            "Reserva confirmada exitosamente. Se han generado {} deudas pendientes (una por tour) con monto total S/ {} y vencimiento el {}",
            num_deudas, &monto_total, fecha_vencimiento_display
        );

        Ok(ConfirmReservaResponse {
            file: self.build_file_response(updated_file, tours_dto).await,
            pago_file_ids,
            monto_total,
            fecha_vencimiento: fecha_vencimiento_display.to_string(),
            mensaje,
        })
    }

    async fn recalculate_file_totals(&self, file_id: i32) -> Result<(), ApplicationError> {
        let all_pagos = self.pago_file_repository.find_all_by_file(file_id).await?;
        let zero = BigDecimal::from(0);

        let monto_total: BigDecimal = all_pagos.iter()
            .filter(|p| p.tipo_registro == "deuda" && p.estado != "cancelado")
            .map(|p| &p.monto_total)
            .fold(zero.clone(), |acc, m| acc + m);

        let monto_pagado: BigDecimal = all_pagos.iter()
            .filter(|p| p.estado != "cancelado")
            .map(|p| {
                if let Some(saldo) = &p.monto_saldo_favor {
                    saldo.clone()
                } else {
                    p.monto_pagado.clone()
                }
            })
            .fold(zero.clone(), |acc, m| acc + m);

        let monto_total_display = monto_total.clone();
        let monto_pagado_display = monto_pagado.clone();
        self.file_repository.update_monto_totals(file_id, monto_total, monto_pagado).await?;
        info!("File {} totals recalculated: monto_total={}, monto_pagado={}", file_id, monto_total_display, monto_pagado_display);

        Ok(())
    }

    async fn recalculate_file_monto_total(&self, file_id: i32) -> Result<(), ApplicationError> {
        let all_pagos = self.pago_file_repository.find_all_by_file(file_id).await?;
        let zero = BigDecimal::from(0);

        let monto_total: BigDecimal = all_pagos.iter()
            .filter(|p| p.tipo_registro == "deuda" && p.estado != "cancelado")
            .map(|p| &p.monto_total)
            .fold(zero.clone(), |acc, m| acc + m);

        let monto_total_display = monto_total.clone();
        self.file_repository.update_monto_total_only(file_id, monto_total).await?;
        info!("File {} monto_total recalculated: monto_total={}", file_id, monto_total_display);

        Ok(())
    }

    /// Redistributes excess payment when a tour's precio_aplicado is reduced below what was already paid.
    /// If sum(monto_pagado) > new_monto_total, moves excess to monto_saldo_favor.
    async fn redistribute_excess_payment(
        &self,
        file_id: i32,
        id_file_tour: i32,
        new_monto_total: &BigDecimal,
        updated_by: i32,
    ) -> Result<(), ApplicationError> {
        let zero = BigDecimal::from(0);
        let tolerancia = BigDecimal::from_str("0.01").unwrap_or_else(|_| BigDecimal::from(0));

        // Get all pagos for this file
        let all_pagos = self.pago_file_repository.find_all_by_file(file_id).await?;

        // Get pagos for this specific tour
        let pagos_del_tour: Vec<_> = all_pagos.iter()
            .filter(|p| p.id_file_tour == Some(id_file_tour)
                && matches!(p.tipo_registro.as_str(), "deuda" | "pago" | "pago_final"))
            .collect();

        // Calculate sum of monto_pagado for this tour
        let monto_pagado_sum: BigDecimal = pagos_del_tour.iter()
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);

        // Calculate excess: sum(monto_pagado) - new_monto_total
        let excess = (&monto_pagado_sum - new_monto_total).max(zero.clone());

        if excess <= tolerancia {
            // No excess to handle
            return Ok(());
        }

        info!("Handling excess payment of {} for tour {} (paid: {}, new_total: {})",
            excess, id_file_tour, monto_pagado_sum, new_monto_total);

        // Move excess to monto_saldo_favor
        self.move_to_saldo_favor(&pagos_del_tour, &excess, updated_by).await?;

        // Recalculate file monto_total after adjustment
        self.recalculate_file_monto_total(file_id).await?;

        Ok(())
    }

    /// Moves excess amount to monto_saldo_favor on the deuda record
    async fn move_to_saldo_favor(
        &self,
        pagos_del_tour: &[&PagoFileModel],
        excess: &BigDecimal,
        updated_by: i32,
    ) -> Result<(), ApplicationError> {
        let zero = BigDecimal::from(0);
        let tolerancia = BigDecimal::from_str("0.01").unwrap_or_else(|_| BigDecimal::from(0));

        if excess <= &tolerancia {
            return Ok(());
        }

        // Find the deuda record to add saldo_favor
        if let Some(deuda) = pagos_del_tour.iter().find(|p| p.tipo_registro == "deuda") {
            let saldo_anterior = deuda.monto_saldo_favor.as_ref().unwrap_or(&zero);
            let nuevo_saldo = saldo_anterior + excess;

            let update = UpdatePagoFileModel {
                monto_saldo_favor: Some(nuevo_saldo),
                saldo_autorizado: Some(true),
                saldo_autorizado_por: Some(updated_by),
                saldo_autorizado_at: Some(chrono::Utc::now()),
                notas: Some(&format!("Saldo a favor por ajuste de precio: {}", excess)),
                ..Default::default()
            };
            self.pago_file_repository.update(deuda.id, update).await?;
            info!("Added {} to monto_saldo_favor on deuda {}", excess, deuda.id);
        }

        Ok(())
    }
}

/// Calcula la fecha de vencimiento según el tipo de vencimiento y la fecha del tour.
/// - "semanal": el domingo de la misma semana del tour
/// - "quincenal": el 15 o el último día válido (máx 30) del mes del tour
/// - "mensual": el último día del mes del tour
fn calcular_fecha_vencimiento(tipo: &str, fecha_tour: NaiveDate) -> NaiveDate {
    match tipo {
        "semanal" => {
            // Domingo de la misma semana. Si ya es domingo, ese mismo día.
            let weekday = fecha_tour.weekday();
            if weekday == chrono::Weekday::Sun {
                fecha_tour
            } else {
                // num_days_from_monday(): Lun=0, Mar=1, ..., Sab=5, Dom=6
                let dias_hasta_domingo = 6 - weekday.num_days_from_monday();
                fecha_tour + chrono::Duration::days(dias_hasta_domingo as i64)
            }
        },
        "quincenal" => {
            let day = fecha_tour.day();
            if day <= 15 {
                // Vence el 15 del mismo mes
                NaiveDate::from_ymd_opt(fecha_tour.year(), fecha_tour.month(), 15)
                    .unwrap_or(fecha_tour)
            } else {
                // Vence el 30 o último día del mes (el menor)
                let ultimo_dia = ultimo_dia_del_mes(fecha_tour);
                let dia_vencimiento = ultimo_dia.min(30);
                if day <= dia_vencimiento {
                    NaiveDate::from_ymd_opt(fecha_tour.year(), fecha_tour.month(), dia_vencimiento)
                        .unwrap_or(fecha_tour)
                } else {
                    // Día 31 → 15 del mes siguiente
                    let (y, m) = if fecha_tour.month() == 12 {
                        (fecha_tour.year() + 1, 1)
                    } else {
                        (fecha_tour.year(), fecha_tour.month() + 1)
                    };
                    NaiveDate::from_ymd_opt(y, m, 15).unwrap_or(fecha_tour)
                }
            }
        },
        // "mensual" y cualquier otro valor → último día del mes
        _ => {
            let ultimo = ultimo_dia_del_mes(fecha_tour);
            NaiveDate::from_ymd_opt(fecha_tour.year(), fecha_tour.month(), ultimo)
                .unwrap_or(fecha_tour)
        },
    }
}

/// Retorna el número del último día del mes de la fecha dada.
fn ultimo_dia_del_mes(fecha: NaiveDate) -> u32 {
    let (y, m) = if fecha.month() == 12 {
        (fecha.year() + 1, 1)
    } else {
        (fecha.year(), fecha.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap_or(fecha)
        .pred_opt()
        .unwrap_or(fecha)
        .day()
}
