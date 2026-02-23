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
use tracing::{info, instrument};

use crate::application::dtos::contabilidad_dto::{
    CancelacionResponse, CancelarFileRequest, CancelarFileTourRequest,
    NoShowResponse, RegistrarNoShowRequest, NoShowFileTourRequest,
    AutorizarNoShowSaldoRequest, SaldoFavorResumen, SaldoFavorDashboard,
    MovimientoSaldoResponse, UsarSaldoFavorRequest,
};
use crate::application::ports::{
    PagoFileRepositoryPort, FileRepositoryPort, FileTourRepositoryPort,
    AgenciaRepositoryPort, TourRepositoryPort, NotificationServicePort,
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
    tour_repo: Arc<dyn TourRepositoryPort>,
    notification_service: Arc<dyn NotificationServicePort>,
    file_entrada_repo: Arc<dyn FileEntradaRepositoryPort>,
    entrada_precio_repo: Arc<dyn EntradaPrecioRepositoryPort>,
    entrada_repo: Arc<dyn EntradaRepositoryPort>,
    file_status_service: Arc<FileStatusService>,
}

impl SaldoFavorService {
    pub fn new(
        pago_file_repo: Arc<dyn PagoFileRepositoryPort>,
        file_repo: Arc<dyn FileRepositoryPort>,
        file_tour_repo: Arc<dyn FileTourRepositoryPort>,
        agencia_repo: Arc<dyn AgenciaRepositoryPort>,
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
            tour_repo,
            notification_service,
            file_entrada_repo,
            entrada_precio_repo,
            entrada_repo,
            file_status_service,
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

        // Obtener todos los pagos del file (deuda + pagos)
        let all_pagos = self.pago_file_repo.find_all_by_file(request.id_file).await?;
        let zero = BigDecimal::from(0);

        // Calcular monto total pagado
        let monto_pagado_total = all_pagos.iter()
            .filter(|p| p.tipo_registro == "pago" || p.tipo_registro == "deuda")
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);

        // El monto pagado completo se convierte en saldo a favor
        let monto_saldo = if monto_pagado_total > zero { Some(monto_pagado_total.clone()) } else { None };

        // Crear registro de cancelación
        let new_record = NewPagoFileModel {
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_total: file.monto_total.clone(),
            monto_pagado: zero.clone(),
            estado: "cancelado",
            fecha_vencimiento: None,
            notas: request.notas.as_deref(),
            created_by,
            id_file_tour: None,
            tipo_registro: "cancelacion",
            monto_saldo_favor: monto_saldo,
            saldo_autorizado: true,
            saldo_autorizado_por: created_by,
            saldo_autorizado_at: Some(chrono::Utc::now()),
            entradas: false,
            entrada_precio: None,
        };

        let record = self.pago_file_repo.create(new_record).await?;

        info!("File {} cancelado. Saldo a favor generado: {:?}", request.id_file, record.monto_saldo_favor);

        // Actualizar file status + cascada a file_tours, guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_status(request.id_file, "cancelado").await?;
        info!("File {} y sus relaciones actualizados a status 'cancelado'", request.id_file);

        // Actualizar monto_total del file a 0 (todo cancelado)
        let mut updated_file = file.clone();
        updated_file.monto_total = zero.clone();
        updated_file.monto_pagado = zero.clone();
        updated_file.updated_at = chrono::Utc::now();
        self.file_repo.update(&updated_file).await?;

        // Notificar
        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "File Cancelado",
            &format!("File #{} cancelado. Saldo a favor: S/ {}", request.id_file,
                record.monto_saldo_favor.as_ref().map(|v| v.to_string()).unwrap_or("0".into())),
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

        // Guard: no permitir cancelar un file_tour ya cancelado o no_show
        if ft.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está cancelado", request.id_file_tour)));
        }
        if ft.status == "no_show" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está marcado como no-show", request.id_file_tour)));
        }

        let file = self.file_repo.find_by_id(ft.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", ft.id_file)))?;

        let all_tours = self.file_tour_repo.find_by_file_with_tour(ft.id_file).await?;

        // Obtener todos los pagos del file
        let all_pagos = self.pago_file_repo.find_all_by_file(ft.id_file).await?;
        let zero = BigDecimal::from(0);

        // Usar los valores REALES de la deuda del tour cancelado (no división proporcional)
        let deuda_tour = all_pagos.iter()
            .find(|p| p.id_file_tour == Some(request.id_file_tour) && p.tipo_registro == "deuda");

        // === LÓGICA BTG/BTP: detectar entradas transferibles ===
        let file_entradas = self.file_entrada_repo.find_by_file_tour(request.id_file_tour).await?;
        let mut monto_btg_btp = zero.clone();
        let mut entradas_btg_btp: Vec<(i32, BigDecimal)> = Vec::new(); // (file_entrada_id, costo)

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

        // Buscar siguiente tour por orden (que no esté cancelado)
        let siguiente_tour = all_tours.iter()
            .find(|t| t.orden > ft.orden && t.status != "cancelado");

        let mut monto_transferido = zero.clone();
        let mut id_file_tour_destino: Option<i32> = None;

        if !entradas_btg_btp.is_empty() {
            if let Some(next_tour) = siguiente_tour {
                // Transferir file_entradas BTG/BTP al siguiente tour
                for (fe_id, _costo) in &entradas_btg_btp {
                    let _ = self.file_entrada_repo.transfer_to_file_tour(*fe_id, next_tour.id).await;
                }

                monto_transferido = monto_btg_btp.clone();
                id_file_tour_destino = Some(next_tour.id);

                // Actualizar la deuda del tour destino: sumar monto BTG/BTP
                let deuda_destino = all_pagos.iter()
                    .find(|p| p.id_file_tour == Some(next_tour.id) && p.tipo_registro == "deuda");

                if let Some(deuda) = deuda_destino {
                    let nuevo_total = &deuda.monto_total + &monto_transferido;
                    let nuevo_pagado = &deuda.monto_pagado + &monto_transferido;
                    let nueva_entrada_precio = deuda.entrada_precio.as_ref()
                        .unwrap_or(&zero).clone() + &monto_transferido;
                    let update_deuda = UpdatePagoFileModel {
                        monto_total: Some(nuevo_total),
                        monto_pagado: Some(nuevo_pagado),
                        entradas: Some(true),
                        entrada_precio: Some(Some(nueva_entrada_precio)),
                        ..Default::default()
                    };
                    let _ = self.pago_file_repo.update(deuda.id, update_deuda).await?;
                    info!("Deuda del tour destino {} actualizada con +{}", next_tour.id, monto_transferido);
                }

                info!(
                    "Entradas BTG/BTP transferidas de file_tour {} a file_tour {}. Monto: {}",
                    request.id_file_tour, next_tour.id, monto_transferido
                );
            }
            // Si no hay siguiente tour, el monto BTG/BTP también va a saldo a favor
        }

        // Saldo a favor = monto pagado de la deuda del tour - monto transferido (BTG/BTP)
        let monto_saldo = if let Some(ref deuda) = deuda_tour {
            if deuda.monto_pagado > zero {
                let saldo_final = &deuda.monto_pagado - &monto_transferido;
                if saldo_final > zero { Some(saldo_final) } else { None }
            } else {
                None
            }
        } else {
            None
        };

        let record = if let Some(deuda) = deuda_tour {
            // Actualizar la deuda existente → convertirla en cancelacion_tour
            let update = UpdatePagoFileModel {
                estado: Some("cancelado"),
                tipo_registro: Some("cancelacion_tour"),
                monto_saldo_favor: monto_saldo,
                saldo_autorizado: Some(true),
                saldo_autorizado_por: created_by,
                saldo_autorizado_at: Some(chrono::Utc::now()),
                notas: request.notas.as_deref(),
                ..Default::default()
            };
            self.pago_file_repo.update(deuda.id, update).await?
        } else {
            // Fallback: si no existe deuda, crear registro nuevo
            let new_record = NewPagoFileModel {
                id_file: ft.id_file,
                id_agencia: file.id_agencia,
                monto_total: zero.clone(),
                monto_pagado: zero.clone(),
                estado: "cancelado",
                fecha_vencimiento: None,
                notas: request.notas.as_deref(),
                created_by,
                id_file_tour: Some(request.id_file_tour),
                tipo_registro: "cancelacion_tour",
                monto_saldo_favor: monto_saldo,
                saldo_autorizado: true,
                saldo_autorizado_por: created_by,
                saldo_autorizado_at: Some(chrono::Utc::now()),
                entradas: false,
                entrada_precio: None,
            };
            self.pago_file_repo.create(new_record).await?
        };

        info!("FileTour {} cancelado. Saldo a favor: {:?}", request.id_file_tour, record.monto_saldo_favor);

        // Actualizar file_tour status a "cancelado" + cascada a guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_tour_status(request.id_file_tour, "cancelado").await?;
        info!("FileTour {} y sus relaciones actualizados a status 'cancelado'", request.id_file_tour);

        // Recalcular monto_total del file desde las deudas activas (no canceladas)
        let all_pagos_updated = self.pago_file_repo.find_all_by_file(ft.id_file).await?;
        let file_monto_total = all_pagos_updated.iter()
            .filter(|p| p.tipo_registro == "deuda")
            .map(|p| &p.monto_total)
            .fold(zero.clone(), |acc, m| acc + m);

        let mut updated_file = file.clone();
        updated_file.monto_total = file_monto_total;
        // No actualizamos monto_pagado en files, se gestiona en pagos_files
        updated_file.monto_pagado = zero.clone();
        updated_file.updated_at = chrono::Utc::now();
        self.file_repo.update(&updated_file).await?;

        // Construir respuesta con info de transferencia
        let mut response = self.pago_to_cancelacion_response(record).await?;
        response.monto_entradas_transferidas = monto_transferido.to_f64().unwrap_or(0.0);
        response.id_file_tour_destino = id_file_tour_destino;

        Ok(response)
    }

    /// Listar cancelaciones de una agencia
    #[instrument(skip(self))]
    pub async fn list_cancelaciones(
        &self,
        id_agencia: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CancelacionResponse>, ApplicationError> {
        let tipos = &["cancelacion", "cancelacion_tour"];

        let records = if let Some(ag_id) = id_agencia {
            self.pago_file_repo.find_by_agencia_tipos(ag_id, tipos, limit, offset).await?
        } else {
            // Para admin: obtener todas
            self.pago_file_repo.find_filtered(None, Some("cancelado"), None, None, limit, offset).await?
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

        let zero = BigDecimal::from(0);

        let new_record = NewPagoFileModel {
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_total: file.monto_total.clone(),
            monto_pagado: zero,
            estado: "no_show",
            fecha_vencimiento: None,
            notas: request.notas.as_deref(),
            created_by,
            id_file_tour: None,
            tipo_registro: "no_show",
            monto_saldo_favor: None,
            saldo_autorizado: false,
            saldo_autorizado_por: None,
            saldo_autorizado_at: None,
            entradas: false,
            entrada_precio: None,
        };

        let record = self.pago_file_repo.create(new_record).await?;
        info!("No-show registrado para file {}", request.id_file);

        // Actualizar file status + cascada a file_tours, guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_status(request.id_file, "no_show").await?;
        info!("File {} y sus relaciones actualizados a status 'no_show'", request.id_file);

        let _ = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "No-Show Registrado",
            &format!("File #{} marcado como no-show. Pendiente de autorización de saldo.", request.id_file),
            NotificationType::Warning,
            NotificationCategory::Financial,
            NotificationPriority::Normal,
            created_by,
        ).await;

        self.pago_to_no_show_response(record).await
    }

    /// Registrar no-show de un file_tour específico
    #[instrument(skip(self))]
    pub async fn registrar_no_show_file_tour(
        &self,
        request: NoShowFileTourRequest,
        created_by: Option<i32>,
    ) -> Result<NoShowResponse, ApplicationError> {
        let ft = self.file_tour_repo.find_by_id(request.id_file_tour).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", request.id_file_tour)))?;

        // Guard: no permitir no-show en un file_tour ya cancelado o no_show
        if ft.status == "cancelado" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está cancelado", request.id_file_tour)));
        }
        if ft.status == "no_show" {
            return Err(ApplicationError::Validation(format!("FileTour {} ya está marcado como no-show", request.id_file_tour)));
        }

        let file = self.file_repo.find_by_id(ft.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", ft.id_file)))?;

        // Obtener todos los pagos del file para encontrar la deuda de este tour
        let all_pagos = self.pago_file_repo.find_all_by_file(ft.id_file).await?;
        
        // Usar monto real de la deuda del tour (no proporcional)
        let deuda_tour = all_pagos.iter()
            .find(|p| p.id_file_tour == Some(request.id_file_tour) && p.tipo_registro == "deuda");
        
        let monto_tour = if let Some(deuda) = deuda_tour {
            deuda.monto_total.clone()
        } else {
            // Fallback: proporcional
            let all_tours = self.file_tour_repo.find_by_file_with_tour(ft.id_file).await?;
            let total_tours = all_tours.len() as i32;
            if total_tours > 0 {
                &file.monto_total / BigDecimal::from(total_tours)
            } else {
                file.monto_total.clone()
            }
        };
        let zero = BigDecimal::from(0);

        let new_record = NewPagoFileModel {
            id_file: ft.id_file,
            id_agencia: file.id_agencia,
            monto_total: monto_tour,
            monto_pagado: zero,
            estado: "no_show",
            fecha_vencimiento: None,
            notas: request.notas.as_deref(),
            created_by,
            id_file_tour: Some(request.id_file_tour),
            tipo_registro: "no_show_tour",
            monto_saldo_favor: None,
            saldo_autorizado: false,
            saldo_autorizado_por: None,
            saldo_autorizado_at: None,
            entradas: false,
            entrada_precio: None,
        };

        let record = self.pago_file_repo.create(new_record).await?;
        info!("No-show registrado para file_tour {}", request.id_file_tour);

        // Actualizar file_tour status a "no_show" + cascada a guias, vehiculos, restaurantes, entradas
        let _ = self.file_status_service.update_file_tour_status(request.id_file_tour, "no_show").await?;
        info!("FileTour {} y sus relaciones actualizados a status 'no_show'", request.id_file_tour);

        self.pago_to_no_show_response(record).await
    }

    /// Listar no-shows de una agencia
    #[instrument(skip(self))]
    pub async fn list_no_shows(
        &self,
        id_agencia: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NoShowResponse>, ApplicationError> {
        let tipos = &["no_show", "no_show_tour"];

        let records = if let Some(ag_id) = id_agencia {
            self.pago_file_repo.find_by_agencia_tipos(ag_id, tipos, limit, offset).await?
        } else {
            self.pago_file_repo.find_filtered(None, Some("no_show"), None, None, limit, offset).await?
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
        id_agencia: i32,
    ) -> Result<SaldoFavorResumen, ApplicationError> {
        let agencia = self.agencia_repo.find_by_id(id_agencia).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id_agencia)))?;

        let zero = BigDecimal::from(0);

        // Obtener cancelaciones autorizadas
        let cancelaciones = self.pago_file_repo
            .find_by_agencia_tipos(id_agencia, &["cancelacion", "cancelacion_tour"], 10000, 0).await?;
        let saldo_cancelaciones = cancelaciones.iter()
            .filter(|p| p.saldo_autorizado)
            .map(|p| p.monto_saldo_favor.as_ref().unwrap_or(&zero))
            .fold(zero.clone(), |acc, m| acc + m);

        // Obtener no-shows autorizados
        let no_shows = self.pago_file_repo
            .find_by_agencia_tipos(id_agencia, &["no_show", "no_show_tour"], 10000, 0).await?;
        let saldo_no_shows = no_shows.iter()
            .filter(|p| p.saldo_autorizado)
            .map(|p| p.monto_saldo_favor.as_ref().unwrap_or(&zero))
            .fold(zero.clone(), |acc, m| acc + m);

        let saldo_generado = &saldo_cancelaciones + &saldo_no_shows;

        // Obtener uso de saldo
        let usos = self.pago_file_repo
            .find_by_agencia_tipos(id_agencia, &["uso_saldo"], 10000, 0).await?;
        let saldo_usado = usos.iter()
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);

        let saldo_disponible = &saldo_generado - &saldo_usado;

        Ok(SaldoFavorResumen {
            id_agencia,
            nombre_agencia: agencia.nombre,
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
        id_agencia: i32,
    ) -> Result<SaldoFavorDashboard, ApplicationError> {
        let resumen = self.get_saldo_agencia(id_agencia).await?;

        let cancelaciones = self.list_cancelaciones(Some(id_agencia), 10, 0).await?;
        let no_shows = self.list_no_shows(Some(id_agencia), 10, 0).await?;
        let movimientos = self.list_movimientos(Some(id_agencia), 20, 0).await?;

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
        let mut resumenes = Vec::new();
        for agencia in agencias {
            if let Ok(resumen) = self.get_saldo_agencia(agencia.id).await {
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
        id_agencia: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MovimientoSaldoResponse>, ApplicationError> {
        let tipos = &["cancelacion", "cancelacion_tour", "no_show", "no_show_tour", "uso_saldo"];

        let records = if let Some(ag_id) = id_agencia {
            self.pago_file_repo.find_by_agencia_tipos(ag_id, tipos, limit, offset).await?
        } else {
            // Admin: filtrar por tipos de registro relevantes
            let all = self.pago_file_repo.find_filtered(None, None, None, None, limit * 2, offset).await?;
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
        let resumen = self.get_saldo_agencia(request.id_agencia).await?;
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
            id_agencia: request.id_agencia,
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
        };

        let record = self.pago_file_repo.create(new_record).await?;
        info!("Saldo a favor usado: S/ {} para file {}", request.monto, request.id_file);

        self.pago_to_movimiento_response(record).await
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    async fn pago_to_cancelacion_response(&self, p: PagoFileModel) -> Result<CancelacionResponse, ApplicationError> {
        let file_code = self.get_file_code(p.id_file).await;
        let agencia_nombre = self.get_agencia_nombre(p.id_agencia).await;
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
            id_agencia: p.id_agencia,
            agencia_nombre,
            id_file_tour: p.id_file_tour,
            tour_nombre,
            monto_total: p.monto_total.to_f64().unwrap_or(0.0),
            monto_saldo_favor: p.monto_saldo_favor.as_ref().and_then(|v| v.to_f64()).unwrap_or(0.0),
            monto_entradas: monto_entradas.to_f64().unwrap_or(0.0),
            tipo_cancelacion: p.tipo_registro,
            notas: p.notas,
            created_at: p.created_at,
            monto_entradas_transferidas: 0.0,
            id_file_tour_destino: None,
        })
    }

    async fn pago_to_no_show_response(&self, p: PagoFileModel) -> Result<NoShowResponse, ApplicationError> {
        let file_code = self.get_file_code(p.id_file).await;
        let agencia_nombre = self.get_agencia_nombre(p.id_agencia).await;
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
            id_agencia: p.id_agencia,
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
            id_agencia: p.id_agencia,
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

    async fn get_agencia_nombre(&self, id_agencia: i32) -> Option<String> {
        self.agencia_repo.find_by_id(id_agencia).await.ok().flatten().map(|a| a.nombre)
    }

    async fn get_tour_nombre_by_file_tour(&self, id_file_tour: i32) -> Option<String> {
        if let Ok(Some(ft)) = self.file_tour_repo.find_by_id(id_file_tour).await {
            if let Ok(Some(tour)) = self.tour_repo.find_by_id(ft.id_tour).await {
                return Some(tour.nombre);
            }
        }
        None
    }

    /// Calcular el monto total de entradas para un file_tour específico.
    /// Suma: entrada_precios.precio × file_entrada.cantidad para cada file_entrada.
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

    /// Calcular el monto total de entradas para un file completo (todos sus file_tours).
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
