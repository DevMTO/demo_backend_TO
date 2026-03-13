//! Servicio de Saldo a Favor
//!
//! Maneja cancelaciones, no-shows y saldo a favor usando pagos_files:
//! - tipo_registro: 'cancelacion', 'cancelacion_tour', 'no_show', 'no_show_tour', 'uso_saldo'
//! - monto_saldo_favor: monto generado como crédito
//! - saldo_autorizado: para no-shows, requiere autorización admin
//!
//! Prioridad de pagos al cancelar:
//! 1. Entradas (se adquieren con el dinero del pago)
//! 2. Restaurantes (menos prioritario)

use std::sync::Arc;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use chrono::{NaiveDate, NaiveTime, Utc};
use tracing::{info, instrument};

use crate::application::dtos::contabilidad_dto::{
    CancelacionResponse, CancelarFileRequest, CancelarFileTourRequest,
    NoShowResponse, RegistrarNoShowRequest, NoShowFileTourRequest,
    AutorizarNoShowSaldoRequest, SaldoFavorResumen, SaldoFavorDashboard,
    MovimientoSaldoResponse, UsarSaldoFavorRequest,
};
use crate::application::ports::{
    PagoFileRepositoryPort, FileRepositoryPort, FileTourRepositoryPort,
    AgenciaRepositoryPort, HotelRepositoryPort, TourRepositoryPort, NotificationServicePort,
    FileEntradaRepositoryPort, EntradaPrecioRepositoryPort, EntradaRepositoryPort,
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::{
    UserRole, NotificationType, NotificationCategory, NotificationPriority,
};
use crate::infrastructure::persistence::models::{
    NewPagoFileModel, UpdatePagoFileModel, PagoFileModel,
};
use crate::application::services::file_status_service::FileStatusService;

use bigdecimal::ToPrimitive;

pub struct SaldoFavorService {
    pago_file_repo: Arc<dyn PagoFileRepositoryPort>,
    file_repo: Arc<dyn FileRepositoryPort>,
    file_tour_repo: Arc<dyn FileTourRepositoryPort>,
    agencia_repo: Arc<dyn AgenciaRepositoryPort>,
    hotel_repo: Arc<dyn HotelRepositoryPort>,
    tour_repo: Arc<dyn TourRepositoryPort>,
    notification_service: Arc<dyn NotificationServicePort>,
    file_entrada_repo: Arc<dyn FileEntradaRepositoryPort>,
    entrada_precio_repo: Arc<dyn EntradaPrecioRepositoryPort>,
    entrada_repo: Arc<dyn EntradaRepositoryPort>,
    file_status_service: Arc<FileStatusService>,
}

/// Resultado de aplicar cancelación/no-show a un file_tour
struct FileTourOperationResult {
    pago: PagoFileModel,
    monto_entradas_transferidas: f64,
    id_file_tour_destino: Option<i32>,
}

impl SaldoFavorService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pago_file_repo: Arc<dyn PagoFileRepositoryPort>,
        file_repo: Arc<dyn FileRepositoryPort>,
        file_tour_repo: Arc<dyn FileTourRepositoryPort>,
        agencia_repo: Arc<dyn AgenciaRepositoryPort>,
        hotel_repo: Arc<dyn HotelRepositoryPort>,
        tour_repo: Arc<dyn TourRepositoryPort>,
        notification_service: Arc<dyn NotificationServicePort>,
        file_entrada_repo: Arc<dyn FileEntradaRepositoryPort>,
        entrada_precio_repo: Arc<dyn EntradaPrecioRepositoryPort>,
        entrada_repo: Arc<dyn EntradaRepositoryPort>,
        file_status_service: Arc<FileStatusService>,
    ) -> Self {
        Self {
            pago_file_repo,
            file_repo,
            file_tour_repo,
            agencia_repo,
            hotel_repo,
            tour_repo,
            notification_service,
            file_entrada_repo,
            entrada_precio_repo,
            entrada_repo,
            file_status_service,
        }
    }

    // ========================================================================
    // VALIDACIÓN TEMPORAL 8:40 AM
    // ========================================================================

    /// Determina si la operación solicitada es válida según la ventana temporal.
    /// - Cancelación: antes de las 8:40 AM (hora Perú, UTC-5) del primer tour activo
    /// - No-show: desde las 8:40 AM hasta antes de medianoche del primer tour activo
    fn validar_ventana_temporal(
        all_tours: &[crate::infrastructure::persistence::models::FileTourWithTourModel],
        operacion: &str,
    ) -> Result<(), ApplicationError> {
        let primera_fecha: NaiveDate = all_tours.iter()
            .filter(|t| t.status != "cancelado" && t.status != "no_show")
            .filter_map(|t| t.fecha_tour)
            .min()
            .ok_or_else(|| ApplicationError::Validation(
                "No se encontró una fecha de tour activa para validar la ventana temporal".into()
            ))?;

        // Hora Perú = UTC - 5
        let now_utc = chrono::Utc::now().naive_utc();
        let peru_offset = chrono::Duration::hours(5);
        let now_peru = now_utc - peru_offset;
        let now_date = now_peru.date();
        let now_time = now_peru.time();

        let cutoff_840 = NaiveTime::from_hms_opt(8, 40, 0).unwrap();

        match operacion {
            "cancelacion" => {
                // Permitir cancelación si estamos antes de las 8:40 AM del día del primer tour
                if now_date > primera_fecha {
                    return Err(ApplicationError::Validation(
                        format!("No se puede cancelar: la fecha del primer tour ({}) ya pasó", primera_fecha)
                    ));
                }
                if now_date == primera_fecha && now_time >= cutoff_840 {
                    return Err(ApplicationError::Validation(
                        format!("No se puede cancelar: ya pasaron las 8:40 AM del día del tour ({}). Use no-show.", primera_fecha)
                    ));
                }
                Ok(())
            }
            "no_show" => {
                // Permitir no-show desde las 8:40 AM hasta antes de medianoche del día del primer tour
                if now_date > primera_fecha {
                    return Err(ApplicationError::Validation(
                        format!("No se puede registrar no-show: la fecha del primer tour ({}) ya pasó", primera_fecha)
                    ));
                }
                if now_date == primera_fecha && now_time < cutoff_840 {
                    return Err(ApplicationError::Validation(
                        format!("No se puede registrar no-show antes de las 8:40 AM. Use cancelación. Fecha tour: {}", primera_fecha)
                    ));
                }
                if now_date < primera_fecha {
                    return Err(ApplicationError::Validation(
                        format!("No se puede registrar no-show antes del día del tour ({}). Use cancelación.", primera_fecha)
                    ));
                }
                Ok(())
            }
            _ => Err(ApplicationError::Validation("Operación no válida".into())),
        }
    }

    // ========================================================================
    // CANCELACIONES
    // ========================================================================

    /// Cancelar un file completo. Todo el monto pagado se convierte en saldo a favor.
    #[instrument(skip(self))]
    pub async fn cancelar_file(
        &self,
        request: CancelarFileRequest,
        created_by: Option<i32>,
    ) -> Result<CancelacionResponse, ApplicationError> {
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;

        // Guard: no permitir cancelar un file ya cancelado o no_show
        if file.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("File {} ya está cancelado", request.id_file)));
        }
        if file.status == "no_show" {
            return Err(ApplicationError::Validation(format!("File {} ya está marcado como no-show", request.id_file)));
        }

        // Validar ventana temporal: cancelación solo antes de 8:40 AM del primer tour
        let all_tours = self.file_tour_repo.find_by_file_with_tour(request.id_file).await?;
        Self::validar_ventana_temporal(&all_tours, "cancelacion")?;

        // Obtener todos los pagos del file (deuda + pagos)
        let all_pagos = self.pago_file_repo.find_all_by_file(request.id_file).await?;
        let zero = BigDecimal::from(0);

        let mut record_to_return = None;
        let mut saldo_total_generado = zero.clone();

        // Identificamos las deudas y pagos que suman al total pagado
        let pagos_a_actualizar: Vec<_> = all_pagos.iter()
            .filter(|p| p.tipo_registro == "pago" || p.tipo_registro == "deuda" || p.tipo_registro == "pago_final")
            .collect();

        if pagos_a_actualizar.is_empty() {
             return Err(ApplicationError::Validation(format!("No se encontró deuda/pago para el file {}", request.id_file)));
        }

        for pago in &pagos_a_actualizar {
            let mut monto_saldo_favor = None;
            let mut saldo_autorizado = false;
            let mut saldo_autorizado_por = None;
            let mut saldo_autorizado_at = None;

            if pago.monto_pagado > zero {
                monto_saldo_favor = Some(pago.monto_pagado.clone());
                saldo_autorizado = true;
                saldo_autorizado_por = created_by;
                saldo_autorizado_at = Some(chrono::Utc::now());
                saldo_total_generado += &pago.monto_pagado;
            }

            let update = UpdatePagoFileModel {
                estado: Some("cancelado"),
                tipo_registro: Some("cancelacion"),
                // notas: request.notas.as_deref(),   // NOTE: este request.notas es para la tabla `files`
                monto_saldo_favor,
                saldo_autorizado: Some(saldo_autorizado),
                saldo_autorizado_por,
                saldo_autorizado_at,
                ..Default::default()
            };

            let updated = self.pago_file_repo.update(pago.id, update).await?;
            if record_to_return.is_none() {
                record_to_return = Some(updated);
            }
        }

        let record = record_to_return.unwrap();

        info!("File {} cancelado. Saldo a favor generado: {:?}", request.id_file, saldo_total_generado);

        // Actualizar file status + cascada a file_tours, guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_status(request.id_file, "cancelado").await?;
        info!("File {} y sus relaciones actualizados a status 'cancelado'", request.id_file);

        let mut updated_file = file.clone();
        updated_file.status = "cancelado".to_string();
        updated_file.updated_at = chrono::Utc::now();
        updated_file.notas = request.notas.clone();
        self.file_repo.update(&updated_file).await?;

        // Notificar
        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File Cancelado",
            &format!("File #{} cancelado. Saldo a favor: S/ {}", request.id_file, saldo_total_generado),
            NotificationType::Warning,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;

        self.pago_to_cancelacion_response(record).await
    }

    /// Cancelar un file_tour específico.
    /// Si tiene entradas BTG/BTP, se transfieren al siguiente tour del file.
    /// El resto del monto pagado se convierte en saldo a favor.
    #[instrument(skip(self))]
    pub async fn cancelar_file_tour(
        &self,
        request: CancelarFileTourRequest,
        created_by: Option<i32>,
    ) -> Result<CancelacionResponse, ApplicationError> {
        let ft = self.file_tour_repo.find_by_id(request.id_file_tour).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;

        if ft.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está cancelado", request.id_file_tour)));
        }
        if ft.status == "no_show" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está marcado como no-show", request.id_file_tour)));
        }

        let file = self.file_repo.find_by_id(ft.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", ft.id_file)))?;

        let all_tours = self.file_tour_repo.find_by_file_with_tour(ft.id_file).await?;

        // Validar ventana temporal: cancelación solo antes de 8:40 AM del primer tour
        Self::validar_ventana_temporal(&all_tours, "cancelacion")?;

        let all_pagos = self.pago_file_repo.find_all_by_file(ft.id_file).await?;

        let result = self.aplicar_cancelacion_no_show_file_tour(
            &ft,
            &file,
            &all_tours,
            &all_pagos,
            request.id_file_tour,
            "cancelado",
            "cancelacion_tour",
            "cancelado",
            None, // Don't update notas in pagos_files for file_tour cancellation
            true,
            created_by,
        ).await?;

        // Actualizar notas del file_tour (JSONB)
        let mut updated_ft = ft.clone();
        updated_ft.notas = request.notas.as_ref().map(|n| serde_json::json!(n));
        self.file_tour_repo.update(&updated_ft).await?;

        info!("FileTour {} cancelado. Saldo a favor: {:?}", request.id_file_tour, result.pago.monto_saldo_favor);

        // Notificar
        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "FileTour Cancelado",
            &format!("FileTour #{} cancelado. Saldo a favor: S/ {}", request.id_file_tour,
                result.pago.monto_saldo_favor.as_ref().map(|v: &BigDecimal| v.to_string()).unwrap_or("0".into())),
            NotificationType::Warning,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;


        let response = self.pago_to_cancelacion_response_with_transfer(
            result.pago,
            result.monto_entradas_transferidas,
            result.id_file_tour_destino,
        ).await?;
        Ok(response)
    }

    /// Listar cancelaciones de una agencia
    #[instrument(skip(self))]
    pub async fn list_cancelaciones(
        &self,
        id_entidad: Option<i32>,
        entidad: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CancelacionResponse>, ApplicationError> {
        let tipos = &["cancelacion", "cancelacion_tour"];

        let records = if let Some(ag_id) = id_entidad {
            self.pago_file_repo.find_by_entidad_tipos(ag_id, entidad, tipos, limit, offset).await?
        } else {
            // Para admin: obtener todas
            self.pago_file_repo.find_filtered(None, None, Some("cancelado"), None, None, limit, offset).await?
                .into_iter()
                .filter(|p| tipos.contains(&p.tipo_registro.as_str()))
                .collect()
        };

        let mut responses = Vec::new();
        for r in records {
            responses.push(self.pago_to_cancelacion_response(r).await?);
        }
        Ok(responses)
    }

    // ========================================================================
    // NO-SHOWS
    // ========================================================================

    /// Registrar no-show de un file completo. No genera saldo automático (requiere autorización).
    #[instrument(skip(self))]
    pub async fn registrar_no_show(
        &self,
        request: RegistrarNoShowRequest,
        created_by: Option<i32>,
    ) -> Result<NoShowResponse, ApplicationError> {
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;

        // Guard: no permitir no-show en un file ya cancelado o no_show
        if file.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("File {} ya está cancelado", request.id_file)));
        }
        if file.status == "no_show" {
            return Err(ApplicationError::Validation(format!("File {} ya está marcado como no-show", request.id_file)));
        }

        // Validar ventana temporal: no-show solo de 8:40 AM a medianoche del primer tour
        let all_tours = self.file_tour_repo.find_by_file_with_tour(request.id_file).await?;
        Self::validar_ventana_temporal(&all_tours, "no_show")?;

        // Obtener todas las deudas y pagos del file
        let all_pagos = self.pago_file_repo.find_all_by_file(request.id_file).await?;
        let pagos_a_actualizar: Vec<_> = all_pagos.iter()
            .filter(|p| p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final")
            .collect();

        if pagos_a_actualizar.is_empty() {
            return Err(ApplicationError::Validation(format!("No se encontró deuda/pago para el file {}", request.id_file)));
        }

        // Actualizar todas las deudas y pagos existentes a no_show
        let mut updated_record = None;
        for pago in &pagos_a_actualizar {
            let update = UpdatePagoFileModel {
                estado: Some("no_show"),
                tipo_registro: Some("no_show"),
                notas: request.notas.as_deref(),
                ..Default::default()
            };
            let updated = self.pago_file_repo.update(pago.id, update).await?;
            info!("Pago {} actualizado a no_show para file {}", pago.id, request.id_file);
            if updated_record.is_none() {
                updated_record = Some(updated);
            }
        }

        let record = updated_record.unwrap();
        info!("No-show registrado para file {}", request.id_file);

        // Actualizar file status + cascada a file_tours, guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_status(request.id_file, "no_show").await?;
        info!("File {} y sus relaciones actualizados a status 'no_show'", request.id_file);

        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "No-Show Registrado",
            &format!("File #{} marcado como no-show. Pendiente de autorización de saldo.", request.id_file),
            NotificationType::Error,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;

        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        let _ = self.notification_service.notify_roles_for_entity(
            vec![UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::HotelesGerente],
            file.id_entidad,
            "No-Show Registrado",
            &format!("File #{} marcado como no-show.", request.id_file),
            NotificationType::Error,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;

        self.pago_to_no_show_response(record).await
    }

    /// Registrar no-show de un file_tour específico
    /// Si tiene entradas BTG/BTP, se transfieren al siguiente tour del file.
    #[instrument(skip(self))]
    pub async fn registrar_no_show_file_tour(
        &self,
        request: NoShowFileTourRequest,
        created_by: Option<i32>,
    ) -> Result<NoShowResponse, ApplicationError> {
        let ft = self.file_tour_repo.find_by_id(request.id_file_tour).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;

        if ft.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está cancelado", request.id_file_tour)));
        }
        if ft.status == "no_show" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está marcado como no-show", request.id_file_tour)));
        }

        let file = self.file_repo.find_by_id(ft.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", ft.id_file)))?;

        let all_tours = self.file_tour_repo.find_by_file_with_tour(ft.id_file).await?;

        // Validar ventana temporal: no-show solo de 8:40 AM a medianoche del primer tour
        Self::validar_ventana_temporal(&all_tours, "no_show")?;

        let all_pagos = self.pago_file_repo.find_all_by_file(ft.id_file).await?;

        let result = self.aplicar_cancelacion_no_show_file_tour(
            &ft,
            &file,
            &all_tours,
            &all_pagos,
            request.id_file_tour,
            "no_show",
            "no_show_tour",
            "no_show",
            request.notas.as_deref(),
            false,
            created_by,
        ).await?;

        info!("No-show registrado para file_tour {}", request.id_file_tour);

        // Notificar
        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "No-Show Registrado",
            &format!("FileTour #{} marcado como no-show. Pendiente de autorización de saldo.", request.id_file_tour),
            NotificationType::Error,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;

        let ft = self.file_tour_repo.find_by_id(request.id_file_tour).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;
        let file = self.file_repo.find_by_id(ft.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", ft.id_file)))?;
        let _ = self.notification_service.notify_roles_for_entity(
            vec![UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::HotelesGerente],
            file.id_entidad,
            "No-Show Registrado",
            &format!("FileTour #{} marcado como no-show.", request.id_file_tour),
            NotificationType::Error,
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await;

        self.pago_to_no_show_response(result.pago).await
    }

    /// Listar no-shows de una agencia
    #[instrument(skip(self))]
    pub async fn list_no_shows(
        &self,
        id_entidad: Option<i32>,
        entidad: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NoShowResponse>, ApplicationError> {
        let tipos = &["no_show", "no_show_tour"];

        let records = if let Some(ag_id) = id_entidad {
            self.pago_file_repo.find_by_entidad_tipos(ag_id, entidad, tipos, limit, offset).await?
        } else {
            self.pago_file_repo.find_filtered(None, None, Some("no_show"), None, None, limit, offset).await?
                .into_iter()
                .filter(|p| tipos.contains(&p.tipo_registro.as_str()))
                .collect()
        };

        let mut responses = Vec::new();
        for r in records {
            responses.push(self.pago_to_no_show_response(r).await?);
        }
        Ok(responses)
    }

    /// Autorizar saldo a favor de un no-show (solo admin/superadmin)
    #[instrument(skip(self))]
    pub async fn autorizar_no_show_saldo(
        &self,
        request: AutorizarNoShowSaldoRequest,
        autorizado_por: i32,
    ) -> Result<NoShowResponse, ApplicationError> {
        let pago = self.pago_file_repo.find_by_id(request.id_pago_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Registro {} no encontrado", request.id_pago_file)))?;

        if !pago.tipo_registro.contains("no_show") {
            return Err(ApplicationError::Validation("Solo se puede autorizar saldo en registros de no-show".into()));
        }

        let monto_saldo = BigDecimal::from_str(&request.monto_saldo_favor.to_string())
            .map_err(|_| ApplicationError::Validation("Monto inválido".into()))?;

        let update = UpdatePagoFileModel {
            monto_saldo_favor: Some(monto_saldo),
            saldo_autorizado: Some(true),
            saldo_autorizado_por: Some(autorizado_por),
            saldo_autorizado_at: Some(chrono::Utc::now()),
            ..Default::default()
        };

        let updated = self.pago_file_repo.update(request.id_pago_file, update).await?;
        info!("Saldo autorizado para no-show {}: S/ {}", request.id_pago_file, request.monto_saldo_favor);

        self.pago_to_no_show_response(updated).await
    }

    // ========================================================================
    // SALDO A FAVOR - RESUMEN Y MOVIMIENTOS
    // ========================================================================

    /// Obtener resumen de saldo a favor de una agencia
    #[instrument(skip(self))]
    pub async fn get_saldo_agencia(
        &self,
        id_entidad: i32,
        entidad: Option<&str>,
    ) -> Result<SaldoFavorResumen, ApplicationError> {
        let nombre_entidad = if entidad == Some("hoteles") {
            self.hotel_repo.find_by_id(id_entidad).await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id_entidad)))?
                .nombre
        } else {
            self.agencia_repo.find_by_id(id_entidad).await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id_entidad)))?
                .nombre
        };

        let zero = BigDecimal::from(0);

        // Obtener cancelaciones autorizadas
        let cancelaciones = self.pago_file_repo
            .find_by_entidad_tipos(id_entidad, entidad, &["cancelacion", "cancelacion_tour"], 10000, 0).await?;
        let saldo_cancelaciones = cancelaciones.iter()
            .filter(|p| p.saldo_autorizado)
            .map(|p| p.monto_saldo_favor.as_ref().unwrap_or(&zero))
            .fold(zero.clone(), |acc, m| acc + m);

        // Obtener no-shows autorizados
        let no_shows = self.pago_file_repo
            .find_by_entidad_tipos(id_entidad, entidad, &["no_show", "no_show_tour"], 10000, 0).await?;
        let saldo_no_shows = no_shows.iter()
            .filter(|p| p.saldo_autorizado)
            .map(|p| p.monto_saldo_favor.as_ref().unwrap_or(&zero))
            .fold(zero.clone(), |acc, m| acc + m);

        let saldo_generado = &saldo_cancelaciones + &saldo_no_shows;

        // Obtener uso de saldo
        let usos = self.pago_file_repo
            .find_by_entidad_tipos(id_entidad, entidad, &["uso_saldo"], 10000, 0).await?;
        let saldo_usado = usos.iter()
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);

        let saldo_disponible = &saldo_generado - &saldo_usado;

        Ok(SaldoFavorResumen {
            id_entidad,
            nombre_agencia: nombre_entidad,
            saldo_generado: saldo_generado.to_f64().unwrap_or(0.0),
            saldo_usado: saldo_usado.to_f64().unwrap_or(0.0),
            saldo_disponible: saldo_disponible.to_f64().unwrap_or(0.0),
            total_cancelaciones: cancelaciones.len() as i32,
            total_no_shows: no_shows.len() as i32,
        })
    }

    /// Dashboard completo de saldo a favor
    #[instrument(skip(self))]
    pub async fn get_dashboard(
        &self,
        id_entidad: i32,
        entidad: Option<&str>,
    ) -> Result<SaldoFavorDashboard, ApplicationError> {
        let resumen = self.get_saldo_agencia(id_entidad, entidad).await?;

        let cancelaciones = self.list_cancelaciones(Some(id_entidad), entidad, 10, 0).await?;
        let no_shows = self.list_no_shows(Some(id_entidad), entidad, 10, 0).await?;
        let movimientos = self.list_movimientos(Some(id_entidad), entidad, 20, 0).await?;

        Ok(SaldoFavorDashboard {
            resumen,
            cancelaciones_recientes: cancelaciones,
            no_shows_recientes: no_shows,
            movimientos_recientes: movimientos,
        })
    }

    /// Listar todos los saldos de todas las agencias (vista admin)
    #[instrument(skip(self))]
    pub async fn list_all_saldos(&self) -> Result<Vec<SaldoFavorResumen>, ApplicationError> {
        let agencias = self.agencia_repo.list(1000, 0).await?;
        let hoteles = self.hotel_repo.list(1000, 0).await?;
        let mut resumenes = Vec::new();
        for agencia in agencias {
            if let Ok(resumen) = self.get_saldo_agencia(agencia.id, Some("agencias")).await {
                if resumen.total_cancelaciones > 0 || resumen.total_no_shows > 0 {
                    resumenes.push(resumen);
                }
            }
        }
        for hotel in hoteles {
            if let Ok(resumen) = self.get_saldo_agencia(hotel.id, Some("hoteles")).await {
                if resumen.total_cancelaciones > 0 || resumen.total_no_shows > 0 {
                    resumenes.push(resumen);
                }
            }
        }
        Ok(resumenes)
    }

    /// Listar movimientos de saldo (cancelaciones, no-shows autorizados, usos)
    #[instrument(skip(self))]
    pub async fn list_movimientos(
        &self,
        id_entidad: Option<i32>,
        entidad: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MovimientoSaldoResponse>, ApplicationError> {
        let tipos = &["cancelacion", "cancelacion_tour", "no_show", "no_show_tour", "uso_saldo"];

        let records = if let Some(ag_id) = id_entidad {
            self.pago_file_repo.find_by_entidad_tipos(ag_id, entidad, tipos, limit, offset).await?
        } else {
            // Admin: filtrar por tipos de registro relevantes
            let all = self.pago_file_repo.find_filtered(None, None, None, None, None, limit * 2, offset).await?;
            all.into_iter()
                .filter(|p| tipos.contains(&p.tipo_registro.as_str()))
                .take(limit as usize)
                .collect()
        };

        let mut responses = Vec::new();
        for r in records {
            responses.push(self.pago_to_movimiento_response(r).await?);
        }
        Ok(responses)
    }

    /// Usar saldo a favor para pagar un file
    #[instrument(skip(self))]
    pub async fn usar_saldo(
        &self,
        request: UsarSaldoFavorRequest,
        created_by: Option<i32>,
    ) -> Result<MovimientoSaldoResponse, ApplicationError> {
        // Verificar saldo disponible
        let resumen = self.get_saldo_agencia(request.id_entidad, None).await?;
        if request.monto > resumen.saldo_disponible {
            return Err(ApplicationError::Validation(
                format!("Saldo insuficiente. Disponible: S/ {:.2}, Solicitado: S/ {:.2}",
                    resumen.saldo_disponible, request.monto)
            ));
        }

        // Verificar que el file existe
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;

        let monto = BigDecimal::from_str(&request.monto.to_string())
            .map_err(|_| ApplicationError::Validation("Monto inválido".into()))?;

        let concepto = request.concepto.as_deref()
            .unwrap_or("Aplicación de saldo a favor");

        let new_record = NewPagoFileModel {
            id_file: request.id_file,
            id_entidad: request.id_entidad,
            entidad: file.entidad.as_deref(),
            monto_total: file.monto_total.clone(),
            monto_pagado: monto,
            estado: "pagado",
            fecha_vencimiento: None,
            notas: Some(concepto),
            created_by,
            id_file_tour: None,
            tipo_registro: "uso_saldo",
            monto_saldo_favor: None,
            saldo_autorizado: false,
            saldo_autorizado_por: None,
            saldo_autorizado_at: None,
            entradas: false,
            entrada_precio: None,
            cuota: None,
            pagado_por: created_by,
            pagado_at: Some(Utc::now()),
        };

        let record = self.pago_file_repo.create(new_record).await?;
        info!("Saldo a favor usado: S/ {} para file {}", request.monto, request.id_file);

        self.pago_to_movimiento_response(record).await
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    /// Procesa la cancelación o no-show de un file_tour.
    ///
    /// # Proceso:
    /// 1. Identifica las entradas BTG/BTP del tour
    /// 2. Si existe un siguiente tour, transfiere las entradas BTG/BTP a ese tour
    /// 3. Actualiza los pagos relacionados (monto_total, saldo a favor)
    /// 4. Actualiza el status del file_tour
    /// 5. Recalcula los totales del file (monto_total, monto_pagado)
    #[allow(clippy::too_many_arguments)]
    async fn aplicar_cancelacion_no_show_file_tour(
        &self,
        ft: &crate::infrastructure::persistence::models::FileTourModel,
        file: &crate::domain::entities::file::File,
        all_tours: &[crate::infrastructure::persistence::models::FileTourWithTourModel],
        all_pagos: &[PagoFileModel],
        id_file_tour: i32,
        estado: &str,
        tipo_registro: &str,
        status_file_tour: &str,
        notas: Option<&str>,
        pasar_a_saldo_favor: bool,
        created_by: Option<i32>,
    ) -> Result<FileTourOperationResult, ApplicationError> {
        let zero = BigDecimal::from(0);
        let tolerancia = BigDecimal::from_str("0.01").unwrap();

        let mut original_precios: std::collections::HashMap<i32, BigDecimal> = std::collections::HashMap::new();
        for t in all_tours {
            original_precios.insert(t.id, t.precio_aplicado.clone().unwrap_or_else(|| zero.clone()));
        }

        let deuda_tour = all_pagos.iter()
            .find(|p| p.id_file_tour == Some(id_file_tour) && p.tipo_registro == "deuda");

        let pagos_del_tour: Vec<_> = all_pagos.iter()
            .filter(|p| p.id_file_tour == Some(id_file_tour) && (p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final"))
            .collect();

        let siguiente_tour = all_tours.iter()
            .find(|t| t.orden > ft.orden && t.status != "cancelado" && t.status != "no_show");

        // Obtener entradas del tour
        let file_entradas = self.file_entrada_repo.find_by_file_tour(id_file_tour).await?;
        
        // Calcular monto de entradas BTG/BTP (boleto turístico)
        let mut monto_btg_btp = zero.clone();
        let mut entradas_btg_btp: Vec<(i32, BigDecimal)> = Vec::new();
        for fe in &file_entradas {
            if let Ok(Some(entrada)) = self.entrada_repo.find_by_id(fe.id_entrada).await {
                if entrada.boleto_turistico {
                    let costo = if let Some(precio_id) = fe.id_entrada_precio {
                        if let Ok(Some(precio)) = self.entrada_precio_repo.find_by_id(precio_id).await {
                            &precio.precio * BigDecimal::from(fe.cantidad)
                        } else {
                            zero.clone()
                        }
                    } else {
                        zero.clone()
                    };
                    monto_btg_btp += &costo;
                    entradas_btg_btp.push((fe.id, costo));
                }
            }
        }

        let mut monto_transferido = zero.clone();
        let mut id_tour_destino: Option<i32> = None;

        // Transferir entradas BTG/BTP al siguiente tour si existe
        if !entradas_btg_btp.is_empty() {
            if let Some(next_tour) = siguiente_tour {
                id_tour_destino = Some(next_tour.id);
                // Transferir las entradas al siguiente tour
                for (fe_id, _costo) in &entradas_btg_btp {
                    let _ = self.file_entrada_repo.transfer_to_file_tour(*fe_id, next_tour.id).await;
                }

                monto_transferido = monto_btg_btp.clone();

                // Actualizar la deuda del tour destino con el monto transferido
                let deuda_destino = all_pagos.iter()
                    .find(|p| p.id_file_tour == Some(next_tour.id) && p.tipo_registro == "deuda");

                if let Some(deuda) = deuda_destino {
                    let nuevo_total = &deuda.monto_total + &monto_transferido;
                    let nueva_entrada_precio = deuda.entrada_precio.as_ref()
                        .unwrap_or(&zero).clone() + &monto_transferido;
                    let update_deuda = UpdatePagoFileModel {
                        monto_total: Some(nuevo_total.clone()),
                        entradas: Some(true),
                        entrada_precio: Some(Some(nueva_entrada_precio)),
                        ..Default::default()
                    };
                    let _ = self.pago_file_repo.update(deuda.id, update_deuda).await?;
                    info!("Deuda del tour destino {} actualizada con +{}", next_tour.id, monto_transferido);

                    // Actualizar monto_total en los demás pagos del tour destino
                    let pagos_destino: Vec<_> = all_pagos.iter()
                        .filter(|p| p.id_file_tour == Some(next_tour.id) && p.id != deuda.id)
                        .collect();
                    for pago in pagos_destino {
                        let update_pago = UpdatePagoFileModel {
                            monto_total: Some(nuevo_total.clone()),
                            ..Default::default()
                        };
                        let _ = self.pago_file_repo.update(pago.id, update_pago).await?;
                        info!("Pago {} del tour destino {} actualizado con monto_total {}", pago.id, next_tour.id, nuevo_total);
                    }
                }

                // Copiar comprobante de pago del tour cancelado al tour destino
                if let Some(deuda) = deuda_tour {
                    if deuda.comprobante_url.is_some() || deuda.comprobante_key.is_some() {
                        if let Some(deuda_dest) = deuda_destino {
                            let update_comprobante = UpdatePagoFileModel {
                                comprobante_url: deuda.comprobante_url.as_deref(),
                                comprobante_key: deuda.comprobante_key.as_deref(),
                                ..Default::default()
                            };
                            let _ = self.pago_file_repo.update(deuda_dest.id, update_comprobante).await?;
                        }
                    }
                }

                // Actualizar precio_aplicado en ambos tours
                let precio_cancelado = ft.precio_aplicado.clone().unwrap_or_else(|| zero.clone());
                let nuevo_precio_cancelado = &precio_cancelado - &monto_transferido;
                let nuevo_precio_cancelado = if nuevo_precio_cancelado > zero { Some(nuevo_precio_cancelado) } else { Some(zero.clone()) };
                self.file_tour_repo.update_precio_aplicado(id_file_tour, nuevo_precio_cancelado.clone()).await?;

                let precio_destino = next_tour.precio_aplicado.clone().unwrap_or_else(|| zero.clone());
                let nuevo_precio_destino = &precio_destino + &monto_transferido;
                self.file_tour_repo.update_precio_aplicado(next_tour.id, Some(nuevo_precio_destino.clone())).await?;

                info!("Entradas BTG/BTP transferidas de file_tour {} a file_tour {}. Monto: {}", id_file_tour, next_tour.id, monto_transferido);
            }
        }

        // Transferir monto_pagado a siguiente(s) tour(s)
        let monto_pagado_transferir = pagos_del_tour.iter()
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);

        let mut monto_restante = monto_pagado_transferir.clone();

        if monto_restante > zero {
            let mut tours_siguientes: Vec<_> = all_tours.iter()
                .filter(|t| t.orden > ft.orden && t.status != "cancelado" && t.status != "no_show")
                .collect();
            tours_siguientes.sort_by(|a, b| a.orden.cmp(&b.orden));

            for tour_sig in tours_siguientes {
                if monto_restante <= zero {
                    break;
                }

                let pagos_tour_sig: Vec<_> = all_pagos.iter()
                    .filter(|p| p.id_file_tour == Some(tour_sig.id) && (p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final"))
                    .collect();

                let _deuda_tour_sig = all_pagos.iter()
                    .find(|p| p.id_file_tour == Some(tour_sig.id) && p.tipo_registro == "deuda");

                let monto_total_sig = original_precios.get(&tour_sig.id).cloned().unwrap_or_else(|| zero.clone());
                // Add BTG/BTP transfer if this tour received the transfer
                let monto_total_sig = if Some(tour_sig.id) == siguiente_tour.map(|t| t.id) {
                    &monto_total_sig + &monto_transferido
                } else {
                    monto_total_sig
                };
                
                let pagado_sig: BigDecimal = pagos_tour_sig.iter()
                    .map(|p| &p.monto_pagado)
                    .fold(zero.clone(), |acc, m| acc + m);

                let pendiente_sig = &monto_total_sig - &pagado_sig;

                if pendiente_sig <= zero {
                    continue;
                }

                let a_transferir = monto_restante.clone().min(pendiente_sig.clone());
                if a_transferir <= zero {
                    continue;
                }

                if let Some(deuda) = _deuda_tour_sig {
                    let base_pagado = deuda.monto_pagado.clone();
                    
                    let base_total = if Some(tour_sig.id) == siguiente_tour.map(|t| t.id) {
                        &deuda.monto_total + &monto_transferido
                    } else {
                        deuda.monto_total.clone()
                    };

                    let nuevo_pagado = &base_pagado + &a_transferir;
                    let estado_deuda_nuevo = if &base_total - &nuevo_pagado <= tolerancia {
                        "pagado"
                    } else {
                        "parcial"
                    };

                    let mut set_notas = format!("Transferido {} de tour cancelado/no-show #{}", a_transferir, id_file_tour);
                    if let Some(ref notas_antiguas) = deuda.notas {
                        if !notas_antiguas.is_empty() {
                            set_notas = format!("{} | {}", notas_antiguas, set_notas);
                        }
                    }

                    let update_deuda = UpdatePagoFileModel {
                        monto_pagado: Some(nuevo_pagado),
                        estado: Some(estado_deuda_nuevo),
                        notas: Some(&set_notas),
                        ..Default::default()
                    };
                    let _ = self.pago_file_repo.update(deuda.id, update_deuda).await?;
                    info!("Pago transferido: S/ {} al tour {} (actualizando deuda {})", a_transferir, tour_sig.id, deuda.id);
                }

                monto_restante -= &a_transferir;
            }
        }

        // Calcular el precio_final para actualizar monto_total en los pagos
        let precio_final = if monto_transferido > zero {
            let original_precio = ft.precio_aplicado.clone().unwrap_or_else(|| zero.clone());
            let precio_after_transfer = &original_precio - &monto_transferido;
            if precio_after_transfer > zero { precio_after_transfer } else { zero.clone() }
        } else {
            ft.precio_aplicado.clone().unwrap_or_else(|| zero.clone())
        };

        // Actualizar los pagos del tour (cambiar estado y tipo_registro)
        let mut record = None;
        let mut dinero_restante_aplicado = false;
        for pago in &pagos_del_tour {
            let dinero_restante = monto_restante.clone();

            let mut update = UpdatePagoFileModel {
                estado: Some(estado),
                tipo_registro: Some(tipo_registro),
                notas,
                monto_total: Some(precio_final.clone()),
                ..Default::default()
            };

            // Si hubo transferencia de monto_pagado, actualizar monto_pagado restando lo transferido
            if monto_transferido > zero {
                update.monto_pagado = Some(zero.clone());
                if let Some(ref entrada_precio) = pago.entrada_precio {
                    let nuevo_entrada_precio = entrada_precio - &monto_transferido;
                    update.entrada_precio = Some(Some( if nuevo_entrada_precio > zero { nuevo_entrada_precio } else { zero.clone() }));
                }
            }

            // Apply saldo_favor to the first payment/deuda (which is the one being canceled)
            if !dinero_restante_aplicado && dinero_restante > zero
                && pago.tipo_registro == "deuda" && pago.id_file_tour == Some(id_file_tour)
            {
                if pasar_a_saldo_favor {
                    update.monto_saldo_favor = Some(dinero_restante.clone());
                    update.saldo_autorizado = Some(true);
                    update.saldo_autorizado_por = created_by;
                    update.saldo_autorizado_at = Some(chrono::Utc::now());
                } else { 
                    update.monto_pagado = Some(dinero_restante.clone());
                }
                dinero_restante_aplicado = true;
            }

            let updated = self.pago_file_repo.update(pago.id, update).await?;
            info!("Pago {} actualizado a {} para file_tour {} (saldo: {:?})", pago.id, tipo_registro, id_file_tour, dinero_restante);
            if record.is_none() {
                record = Some(updated);
            }
        }

        let record = record.unwrap();

        // Actualizar el status del file_tour (cancelado/no_show)
        let _ = self.file_status_service.update_file_tour_status(id_file_tour, status_file_tour).await?;
        info!("FileTour {} y sus relaciones actualizadas a status '{}'", id_file_tour, status_file_tour);

        // Recalcular totales del file (monto_total y monto_pagado)
        let all_pagos = self.pago_file_repo.find_all_by_file(file.id).await?;
        let file_monto_total = all_pagos.iter()
            .filter(|p| p.tipo_registro == "deuda")
            .map(|p| &p.monto_total)
            .fold(zero.clone(), |acc, m| acc + m);
        let active_tour_ids: Vec<Option<i32>> = all_pagos.iter()
            .filter(|p| p.tipo_registro == "deuda")
            .map(|p| p.id_file_tour)
            .collect();
        let file_monto_pagado = all_pagos.iter()
            .filter(|p| {
                (p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final" || p.tipo_registro == "uso_saldo")
                    && active_tour_ids.contains(&p.id_file_tour)
            })
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);
        let mut updated_file = file.clone();
        updated_file.monto_total = file_monto_total;
        updated_file.monto_pagado = file_monto_pagado;
        updated_file.updated_at = chrono::Utc::now();
        self.file_repo.update(&updated_file).await?;

        // Auto-cancelar el file si todos los tours están cancelados o no_show
        let all_tours_refreshed = self.file_tour_repo.find_by_file_with_tour(file.id).await?;
        let all_done = !all_tours_refreshed.is_empty() && all_tours_refreshed.iter()
            .all(|t| t.status == "cancelado" || t.status == "no_show");
        if all_done && updated_file.status != "cancelado" && updated_file.status != "no_show" {
            let final_status = if all_tours_refreshed.iter().all(|t| t.status == "cancelado") {
                "cancelado"
            } else {
                "no_show"
            };
            self.file_repo.update_status(file.id, final_status).await?;
            info!("File {} auto-actualizado a '{}' porque todos los tours están terminados", file.id, final_status);
        }

        Ok(FileTourOperationResult {
            pago: record,
            monto_entradas_transferidas: monto_transferido.to_f64().unwrap_or(0.0),
            id_file_tour_destino: id_tour_destino,
        })
    }

    async fn pago_to_cancelacion_response(&self, p: PagoFileModel) -> Result<CancelacionResponse, ApplicationError> {
        self.pago_to_cancelacion_response_with_transfer(p, 0.0, None).await
    }

    async fn pago_to_cancelacion_response_with_transfer(
        &self,
        p: PagoFileModel,
        monto_entradas_transferidas: f64,
        id_file_tour_destino: Option<i32>,
    ) -> Result<CancelacionResponse, ApplicationError> {
        let file_code = self.get_file_code(p.id_file).await;
        let agencia_nombre = self.get_entity_nombre(p.id_entidad, p.entidad.as_deref()).await;
        let tour_nombre = if let Some(ft_id) = p.id_file_tour {
            self.get_tour_nombre_by_file_tour(ft_id).await
        } else { None };

        // Calcular monto de entradas según el tipo de cancelación
        let monto_entradas = if let Some(ft_id) = p.id_file_tour {
            self.calcular_monto_entradas_tour(ft_id).await
        } else {
            self.calcular_monto_entradas_file(p.id_file).await
        };

        Ok(CancelacionResponse {
            id: p.id,
            id_file: p.id_file,
            file_code,
            id_entidad: p.id_entidad,
            agencia_nombre,
            id_file_tour: p.id_file_tour,
            tour_nombre,
            monto_total: p.monto_total.to_f64().unwrap_or(0.0),
            monto_saldo_favor: p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0),
            monto_entradas: monto_entradas.to_f64().unwrap_or(0.0),
            tipo_cancelacion: p.tipo_registro,
            notas: p.notas,
            created_at: p.created_at,
            monto_entradas_transferidas,
            id_file_tour_destino,
        })
    }

    async fn pago_to_no_show_response(&self, p: PagoFileModel) -> Result<NoShowResponse, ApplicationError> {
        let file_code = self.get_file_code(p.id_file).await;
        let agencia_nombre = self.get_entity_nombre(p.id_entidad, p.entidad.as_deref()).await;
        let tour_nombre = if let Some(ft_id) = p.id_file_tour {
            self.get_tour_nombre_by_file_tour(ft_id).await
        } else { None };

        // Calcular monto de entradas según el tipo de no-show
        let monto_entradas = if let Some(ft_id) = p.id_file_tour {
            self.calcular_monto_entradas_tour(ft_id).await
        } else {
            self.calcular_monto_entradas_file(p.id_file).await
        };

        Ok(NoShowResponse {
            id: p.id,
            id_file: p.id_file,
            file_code,
            id_entidad: p.id_entidad,
            agencia_nombre,
            id_file_tour: p.id_file_tour,
            tour_nombre,
            monto_total: p.monto_total.to_f64().unwrap_or(0.0),
            monto_saldo_favor: p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0),
            monto_entradas: monto_entradas.to_f64().unwrap_or(0.0),
            saldo_autorizado: p.saldo_autorizado,
            saldo_autorizado_por: p.saldo_autorizado_por,
            saldo_autorizado_at: p.saldo_autorizado_at,
            notas: p.notas,
            created_at: p.created_at,
        })
    }

    async fn pago_to_movimiento_response(&self, p: PagoFileModel) -> Result<MovimientoSaldoResponse, ApplicationError> {
        let file_code = self.get_file_code(p.id_file).await;

        let (tipo, concepto, monto) = match p.tipo_registro.as_str() {
            "cancelacion" => ("credito".to_string(), "Cancelación de file".to_string(),
                p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0)),
            "cancelacion_tour" => ("credito".to_string(), "Cancelación de tour".to_string(),
                p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0)),
            "no_show" => ("credito".to_string(), "No-show de file".to_string(),
                p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0)),
            "no_show_tour" => ("credito".to_string(), "No-show de tour".to_string(),
                p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0)),
            "uso_saldo" => ("debito".to_string(),
                p.notas.clone().unwrap_or("Uso de saldo".to_string()),
                p.monto_pagado.to_f64().unwrap_or(0.0)),
            _ => ("otro".to_string(), p.tipo_registro.clone(), 0.0),
        };

        Ok(MovimientoSaldoResponse {
            id: p.id,
            id_file: p.id_file,
            file_code,
            id_entidad: p.id_entidad,
            id_file_tour: p.id_file_tour,
            tipo,
            concepto,
            monto,
            notas: p.notas,
            created_at: p.created_at,
        })
    }

    async fn get_file_code(&self, id_file: i32) -> Option<String> {
        self.file_repo.find_by_id(id_file).await.ok().flatten().and_then(|f| f.file_code)
    }

    async fn get_entity_nombre(&self, id_entidad: i32, entidad: Option<&str>) -> Option<String> {
        if entidad == Some("hoteles") {
            self.hotel_repo.find_by_id(id_entidad).await.ok().flatten().map(|h| h.nombre)
        } else {
            self.agencia_repo.find_by_id(id_entidad).await.ok().flatten().map(|a| a.nombre)
        }
    }

    async fn get_tour_nombre_by_file_tour(&self, id_file_tour: i32) -> Option<String> {
        if let Ok(Some(ft)) = self.file_tour_repo.find_by_id(id_file_tour).await {
            if let Ok(Some(tour)) = self.tour_repo.find_by_id(ft.id_tour).await {
                return Some(tour.nombre);
            }
        }
        None
    }

    async fn calcular_monto_entradas_tour(&self, id_file_tour: i32) -> BigDecimal {
        let zero = BigDecimal::from(0);
        let entradas = match self.file_entrada_repo.find_by_file_tour(id_file_tour).await {
            Ok(es) => es,
            Err(_) => return zero,
        };

        let mut total = zero;
        for fe in &entradas {
            if let Some(precio_id) = fe.id_entrada_precio {
                if let Ok(Some(precio)) = self.entrada_precio_repo.find_by_id(precio_id).await {
                    total += &precio.precio * BigDecimal::from(fe.cantidad);
                }
            }
        }
        total
    }

    async fn calcular_monto_entradas_file(&self, id_file: i32) -> BigDecimal {
        let zero = BigDecimal::from(0);
        let tours = match self.file_tour_repo.find_by_file_with_tour(id_file).await {
            Ok(ts) => ts,
            Err(_) => return zero,
        };

        let mut total = zero;
        for ft in &tours {
            total += self.calcular_monto_entradas_tour(ft.id).await;
        }
        total
    }
}
