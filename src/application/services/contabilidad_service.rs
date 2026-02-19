//! Servicios de contabilidad
//!
//! Incluye:
//! - ContabilidadService: Servicio principal que orquesta operaciones de contabilidad
//! - Dashboard para agencias
//! - Gestion de pagos de files y pagos a proveedores

use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use std::str::FromStr;
use tracing::{info, instrument, warn};

use crate::application::dtos::contabilidad_dto::{
    AgenciaContabilidadDashboard,
    PagoFileResponse, PagoProveedorResponse,
    RegistrarPagoFileRequest, VerificarPagoFileRequest,
    CreatePagoProveedorRequest, MarcarPagoProveedorPagadoRequest,
};
use crate::application::ports::{
    PagoFileRepositoryPort, PagoProveedorRepositoryPort,
    AgenciaRepositoryPort, FileRepositoryPort, NotificationServicePort,
    FileTourRepositoryPort, TourRepositoryPort,
    TransporteRepositoryPort, RestauranteRepositoryPort, GuiaRepositoryPort,
    UserRepositoryPort, PersonaRepositoryPort,
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::{
    UserRole, NotificationType, NotificationCategory, NotificationPriority,
};
use crate::infrastructure::persistence::models::{
    NewPagoProveedorModel,
    NewPagoFileModel,
    UpdatePagoFileModel, UpdatePagoProveedorModel,
    PagoFileModel, PagoProveedorModel,
};

// ============================================================================
// CONTABILIDAD SERVICE
// ============================================================================

pub struct ContabilidadService {
    pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
    pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    file_repository: Arc<dyn FileRepositoryPort>,
    notification_service: Arc<dyn NotificationServicePort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    tour_repository: Arc<dyn TourRepositoryPort>,
    transporte_repository: Arc<dyn TransporteRepositoryPort>,
    restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
    guia_repository: Arc<dyn GuiaRepositoryPort>,
    user_repository: Arc<dyn UserRepositoryPort>,
    persona_repository: Arc<dyn PersonaRepositoryPort>,
}

impl ContabilidadService {
    pub fn new(
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
        notification_service: Arc<dyn NotificationServicePort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        tour_repository: Arc<dyn TourRepositoryPort>,
        transporte_repository: Arc<dyn TransporteRepositoryPort>,
        restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
        guia_repository: Arc<dyn GuiaRepositoryPort>,
        user_repository: Arc<dyn UserRepositoryPort>,
        persona_repository: Arc<dyn PersonaRepositoryPort>,
    ) -> Self {
        Self {
            pago_file_repository,
            pago_proveedor_repository,
            agencia_repository,
            file_repository,
            notification_service,
            file_tour_repository,
            tour_repository,
            transporte_repository,
            restaurante_repository,
            guia_repository,
            user_repository,
            persona_repository,
        }
    }

    // ========================================================================
    // DASHBOARDS
    // ========================================================================

    /// Obtiene el dashboard de contabilidad para una agencia
    #[instrument(skip(self))]
    pub async fn get_agencia_dashboard(&self, id_agencia: i32) -> Result<AgenciaContabilidadDashboard, ApplicationError> {
        // Verificar que la agencia existe
        let agencia = self.agencia_repository
            .find_by_id(id_agencia)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id_agencia)))?;
        
        // Obtener todos los pagos de files de esta agencia
        let all_pagos = self.pago_file_repository
            .find_by_agencia(id_agencia, 1000, 0)
            .await?;
        
        // Excluir pagos de files cancelados/no_show y registros que no sean deuda/pago
        let pagos: Vec<_> = all_pagos.into_iter()
            .filter(|p| p.estado != "cancelado" && p.estado != "no_show"
                && (p.tipo_registro == "deuda" || p.tipo_registro == "pago"))
            .collect();
        
        // Agrupar por id_file para calcular totales correctos
        let mut file_groups: std::collections::HashMap<i32, Vec<PagoFileModel>> = std::collections::HashMap::new();
        for p in pagos {
            file_groups.entry(p.id_file).or_default().push(p);
        }
        
        let zero = BigDecimal::from_str("0").unwrap();
        let mut global_monto_total = zero.clone();
        let mut global_monto_pagado = zero.clone();
        let mut files_pendientes: Vec<PagoFileResponse> = Vec::new();
        let mut ultimos_pagos: Vec<PagoFileResponse> = Vec::new();
        
        for (_id_file, file_pagos) in &file_groups {
            // monto_total: del registro original (deuda, monto_pagado=0)
            let monto_total = file_pagos.iter()
                .find(|p| p.monto_pagado == zero)
                .map(|p| p.monto_total.clone())
                .unwrap_or_else(|| file_pagos[0].monto_total.clone());
            
            // monto_pagado: suma de todos los montos pagados
            let monto_pagado_file = file_pagos.iter()
                .map(|p| &p.monto_pagado)
                .fold(zero.clone(), |acc, m| acc + m);
            
            let monto_pendiente_file = &monto_total - &monto_pagado_file;
            
            global_monto_total += &monto_total;
            global_monto_pagado += &monto_pagado_file;
            
            // Usar el registro original (deuda) como base
            let base = file_pagos.iter()
                .find(|p| p.monto_pagado == zero)
                .cloned()
                .unwrap_or_else(|| file_pagos[0].clone());
            
            // Determinar estado global del file
            let tolerancia = BigDecimal::from_str("0.01").unwrap();
            let is_fully_paid = monto_pendiente_file <= tolerancia;
            let has_partial = monto_pagado_file > zero;
            let has_verified = file_pagos.iter().any(|p| p.estado == "verificado");
            
            let overall_estado = if has_verified {
                "verificado"
            } else if is_fully_paid {
                "pagado"
            } else if has_partial {
                "parcial"
            } else {
                "pendiente"
            };
            
            // Construir modelo con valores agregados
            let mut aggregated = base.clone();
            aggregated.monto_total = monto_total;
            aggregated.monto_pagado = monto_pagado_file.clone();
            aggregated.estado = overall_estado.to_string();
            
            let response = self.pago_file_to_response_with_calculated_pending(
                aggregated,
                Some(agencia.nombre.clone()),
                Some(monto_pendiente_file.clone()),
            ).await;
            
            if overall_estado == "pendiente" || overall_estado == "parcial" || overall_estado == "vencido" {
                files_pendientes.push(response);
            } else {
                if ultimos_pagos.len() < 10 {
                    ultimos_pagos.push(response);
                }
            }
        }
        
        let monto_pendiente = &global_monto_total - &global_monto_pagado;
        
        Ok(AgenciaContabilidadDashboard {
            id_agencia,
            nombre_agencia: agencia.nombre,
            total_files: file_groups.len() as i32,
            monto_total_files: global_monto_total,
            monto_pagado: global_monto_pagado,
            monto_pendiente,
            pago_anticipado: agencia.pago_anticipado,
            dias_pago_anticipado: agencia.dias_pago_anticipado,
            files_pendientes,
            ultimos_pagos,
        })
    }

    // ========================================================================
    // PAGOS DE FILES
    // ========================================================================

    /// Lista pagos de files con filtros
    #[instrument(skip(self))]
    pub async fn list_pagos_files(
        &self,
        id_agencia: Option<i32>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PagoFileResponse>, i64), ApplicationError> {
        let pagos = self.pago_file_repository
            .find_filtered(id_agencia, estado, fecha_desde, fecha_hasta, limit, offset)
            .await?;

        let total = self.pago_file_repository
            .count_filtered(id_agencia, estado, fecha_desde, fecha_hasta)
            .await?;

        // Group pagos by file to calculate cumulative monto_pagado per file
        let mut file_ids: Vec<i32> = pagos.iter().map(|p| p.id_file).collect();
        file_ids.sort_unstable();
        file_ids.dedup();

        // Fetch all pagos for these files to calculate totals
        let mut file_pagos_totals: std::collections::HashMap<i32, (BigDecimal, BigDecimal)> = std::collections::HashMap::new();
        for id_file in file_ids {
            if let Ok(all_pagos) = self.pago_file_repository.find_all_by_file(id_file).await {
                let monto_total = all_pagos.iter()
                    .find(|p| p.monto_pagado == BigDecimal::from_str("0").unwrap())
                    .map(|p| p.monto_total.clone())
                    .unwrap_or_else(|| all_pagos.first().map(|p| p.monto_total.clone()).unwrap_or_else(|| BigDecimal::from_str("0").unwrap()));

                let monto_pagado_total = all_pagos.iter()
                    .map(|p| &p.monto_pagado)
                    .fold(BigDecimal::from_str("0").unwrap(), |acc, m| acc + m);

                file_pagos_totals.insert(id_file, (monto_total, monto_pagado_total));
            }
        }

        let mut responses = Vec::new();
        for p in pagos {
            let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(p.id_agencia).await {
                Some(agencia.nombre)
            } else {
                None
            };

            // Get the calculated totals for this file
            let (monto_total, monto_pagado_total) = file_pagos_totals
                .get(&p.id_file)
                .cloned()
                .unwrap_or_else(|| (p.monto_total.clone(), p.monto_pagado.clone()));

            let monto_pendiente = &monto_total - &monto_pagado_total;

            responses.push(self.pago_file_to_response_with_calculated_pending(p, agencia_nombre, Some(monto_pendiente)).await);
        }

        Ok((responses, total))
    }

    /// Registrar pago de file (agencia sube comprobante)
    ///
    /// Lógica de distribución:
    /// - Primer pago: prioriza cubrir entradas de todos los file_tours con entradas=true,
    ///   luego distribuye el sobrante a las deudas restantes.
    /// - Segundo+ pagos: llena deudas con monto_pagado=0 primero (prioridad entradas),
    ///   luego para deudas parcialmente pagadas crea nuevos registros tipo "pago".
    pub async fn registrar_pago_file(
        &self,
        request: RegistrarPagoFileRequest,
        created_by: Option<i32>,
        comprobante_url: Option<String>,
        comprobante_key: Option<String>,
    ) -> Result<PagoFileResponse, ApplicationError> {
        // 1. Obtener el registro original para conocer el file
        let pago_original = self.pago_file_repository
            .find_by_id(request.id_pago_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", request.id_pago_file)))?;
            
        let id_file = pago_original.id_file;
        let id_agencia = pago_original.id_agencia;

        // 2. Obtener TODAS las deudas (tipo_registro=deuda) y pagos del file
        let all_records = self.pago_file_repository.find_all_by_file(id_file).await?;
        let zero = BigDecimal::from_str("0").unwrap();
        let tolerancia = BigDecimal::from_str("0.01").unwrap();
        
        // Calcular monto total del file (suma de todas las deudas por tour)
        let monto_total_file: BigDecimal = all_records.iter()
            .filter(|p| p.tipo_registro == "deuda")
            .map(|p| &p.monto_total)
            .fold(zero.clone(), |acc, m| acc + m);
        
        // Calcular cuánto se ha pagado en total (deudas con monto_pagado>0 + registros tipo pago)
        let monto_pagado_global: BigDecimal = all_records.iter()
            .filter(|p| p.tipo_registro == "deuda" || p.tipo_registro == "pago")
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);
        
        // 3. Validar que hay monto pendiente
        let monto_pendiente_global = &monto_total_file - &monto_pagado_global;
        
        if request.monto > &monto_pendiente_global + &tolerancia {
            return Err(ApplicationError::Validation(
                format!("El monto S/ {} excede el pendiente S/ {}", request.monto, monto_pendiente_global)
            ));
        }
        
        // 4. Distribuir el pago entre las deudas por tour
        let mut monto_restante = request.monto.clone();
        let mut ultimo_pago_id: Option<i32> = None;
        
        // Recopilar deudas pendientes y parciales
        let deudas: Vec<_> = all_records.iter()
            .filter(|p| p.tipo_registro == "deuda")
            .cloned()
            .collect();
        
        // Calcular cuánto se ha acumulado en pagos por cada deuda (por file_tour)
        let pagos_por_tour: Vec<_> = all_records.iter()
            .filter(|p| p.tipo_registro == "pago")
            .cloned()
            .collect();
        
        // Para cada deuda, calcular su monto_pagado real (monto de la deuda + pagos asociados a ese tour)
        struct DeudaInfo {
            deuda_id: i32,
            id_file_tour: Option<i32>,
            monto_total: BigDecimal,
            monto_pagado_acum: BigDecimal,
            monto_pendiente: BigDecimal,
            entradas: bool,
            entrada_precio: Option<BigDecimal>,
        }
        
        let mut deuda_infos: Vec<DeudaInfo> = Vec::new();
        for d in &deudas {
            // Suma del monto_pagado de la deuda + pagos tipo "pago" asociados al mismo file_tour
            let pagado_en_deuda = d.monto_pagado.clone();
            let pagado_en_pagos: BigDecimal = pagos_por_tour.iter()
                .filter(|p| p.id_file_tour == d.id_file_tour)
                .map(|p| &p.monto_pagado)
                .fold(zero.clone(), |acc, m| acc + m);
            let acumulado = &pagado_en_deuda + &pagado_en_pagos;
            let pendiente = &d.monto_total - &acumulado;
            
            deuda_infos.push(DeudaInfo {
                deuda_id: d.id,
                id_file_tour: d.id_file_tour,
                monto_total: d.monto_total.clone(),
                monto_pagado_acum: acumulado,
                monto_pendiente: if pendiente > zero { pendiente } else { zero.clone() },
                entradas: d.entradas,
                entrada_precio: d.entrada_precio.clone(),
            });
        }
        
        // FASE 1: Priorizar deudas con entradas=true y monto_pagado_acum=0
        // Cubrir primero el costo de entradas, luego el resto
        let mut deudas_con_entradas_pendientes: Vec<&mut DeudaInfo> = deuda_infos.iter_mut()
            .filter(|d| d.entradas && d.monto_pendiente > zero)
            .collect();
        // Ordenar: primero las que tienen monto_pagado_acum=0
        deudas_con_entradas_pendientes.sort_by(|a, b| a.monto_pagado_acum.cmp(&b.monto_pagado_acum));
        
        for d in &mut deudas_con_entradas_pendientes {
            if monto_restante <= zero {
                break;
            }
            
            let a_pagar = monto_restante.clone().min(d.monto_pendiente.clone());
            if a_pagar <= zero {
                continue;
            }
            
            // Determinar si actualizar la deuda directamente o crear nuevo registro
            if d.monto_pagado_acum == zero {
                // Deuda sin pagos: actualizar monto_pagado directamente en la deuda
                let nuevo_pagado = &d.monto_pagado_acum + &a_pagar;
                let pendiente_tras_pago = &d.monto_total - &nuevo_pagado;
                let estado = if pendiente_tras_pago <= tolerancia { "pagado" } else { "parcial" };
                
                let update = UpdatePagoFileModel {
                    monto_pagado: Some(nuevo_pagado.clone()),
                    estado: Some(estado),
                    comprobante_url: comprobante_url.as_deref(),
                    comprobante_key: comprobante_key.as_deref(),
                    ..Default::default()
                };
                let updated = self.pago_file_repository.update(d.deuda_id, update).await?;
                ultimo_pago_id = Some(updated.id);
            } else {
                // Deuda con pagos previos: crear nuevo registro tipo "pago"
                let pending_after = &d.monto_pendiente - &a_pagar;
                let estado = if pending_after <= tolerancia { "pagado" } else { "parcial" };
                let new_pago = NewPagoFileModel {
                    id_file,
                    id_agencia,
                    monto_total: d.monto_total.clone(),
                    monto_pagado: a_pagar.clone(),
                    estado,
                    fecha_vencimiento: None,
                    notas: request.notas.as_deref(),
                    created_by,
                    id_file_tour: d.id_file_tour,
                    tipo_registro: "pago",
                    monto_saldo_favor: None,
                    saldo_autorizado: false,
                    saldo_autorizado_por: None,
                    saldo_autorizado_at: None,
                    entradas: false,
                    entrada_precio: None,
                };
                let pago = self.pago_file_repository.create(new_pago).await?;
                // Adjuntar comprobante
                if comprobante_url.is_some() || comprobante_key.is_some() {
                    let upd = UpdatePagoFileModel {
                        comprobante_url: comprobante_url.as_deref(),
                        comprobante_key: comprobante_key.as_deref(),
                        ..Default::default()
                    };
                    let _ = self.pago_file_repository.update(pago.id, upd).await?;
                }
                ultimo_pago_id = Some(pago.id);
            }
            
            monto_restante -= &a_pagar;
            d.monto_pagado_acum += &a_pagar;
            d.monto_pendiente -= &a_pagar;
        }
        
        // FASE 2: Distribuir sobrante en deudas sin entradas (primero las de monto_pagado=0)
        let mut deudas_sin_entradas: Vec<&mut DeudaInfo> = deuda_infos.iter_mut()
            .filter(|d| !d.entradas && d.monto_pendiente > zero)
            .collect();
        deudas_sin_entradas.sort_by(|a, b| a.monto_pagado_acum.cmp(&b.monto_pagado_acum));
        
        // También incluir deudas con entradas que aún tengan pendiente (entradas ya cubiertas, falta el resto)
        // Re-iterar over all deudas con pendiente
        drop(deudas_sin_entradas);
        let mut deudas_restantes: Vec<&mut DeudaInfo> = deuda_infos.iter_mut()
            .filter(|d| d.monto_pendiente > zero)
            .collect();
        deudas_restantes.sort_by(|a, b| a.monto_pagado_acum.cmp(&b.monto_pagado_acum));
        
        for d in &mut deudas_restantes {
            if monto_restante <= zero {
                break;
            }
            
            let a_pagar = monto_restante.clone().min(d.monto_pendiente.clone());
            if a_pagar <= zero {
                continue;
            }
            
            if d.monto_pagado_acum == zero {
                // Actualizar la deuda directamente
                let nuevo_pagado = a_pagar.clone();
                let pendiente_tras_pago = &d.monto_total - &nuevo_pagado;
                let estado = if pendiente_tras_pago <= tolerancia { "pagado" } else { "parcial" };
                
                let update = UpdatePagoFileModel {
                    monto_pagado: Some(nuevo_pagado),
                    estado: Some(estado),
                    comprobante_url: comprobante_url.as_deref(),
                    comprobante_key: comprobante_key.as_deref(),
                    ..Default::default()
                };
                let updated = self.pago_file_repository.update(d.deuda_id, update).await?;
                ultimo_pago_id = Some(updated.id);
            } else {
                // Crear nuevo registro de pago
                let pending_after = &d.monto_pendiente - &a_pagar;
                let estado = if pending_after <= tolerancia { "pagado" } else { "parcial" };
                let new_pago = NewPagoFileModel {
                    id_file,
                    id_agencia,
                    monto_total: d.monto_total.clone(),
                    monto_pagado: a_pagar.clone(),
                    estado,
                    fecha_vencimiento: None,
                    notas: request.notas.as_deref(),
                    created_by,
                    id_file_tour: d.id_file_tour,
                    tipo_registro: "pago",
                    monto_saldo_favor: None,
                    saldo_autorizado: false,
                    saldo_autorizado_por: None,
                    saldo_autorizado_at: None,
                    entradas: false,
                    entrada_precio: None,
                };
                let pago = self.pago_file_repository.create(new_pago).await?;
                if comprobante_url.is_some() || comprobante_key.is_some() {
                    let upd = UpdatePagoFileModel {
                        comprobante_url: comprobante_url.as_deref(),
                        comprobante_key: comprobante_key.as_deref(),
                        ..Default::default()
                    };
                    let _ = self.pago_file_repository.update(pago.id, upd).await?;
                }
                ultimo_pago_id = Some(pago.id);
            }
            
            monto_restante -= &a_pagar;
            d.monto_pagado_acum += &a_pagar;
            d.monto_pendiente -= &a_pagar;
        }
        
        // 5. Obtener el último registro afectado para respuesta
        let pago_id = ultimo_pago_id.unwrap_or(pago_original.id);
        let pago_registrado = self.pago_file_repository.find_by_id(pago_id).await?
            .ok_or_else(|| ApplicationError::NotFound("Pago no encontrado después de registrar".into()))?;
        
        let nuevo_total_pagado = &monto_pagado_global + &request.monto;
        let deuda_saldada = &nuevo_total_pagado >= &(&monto_total_file - &tolerancia);
        
        info!("Pago distribuido para file {}: S/ {} (total pagado: S/ {}, total deuda: S/ {})", 
            id_file, request.monto, nuevo_total_pagado, monto_total_file);

        // 6. Preparar respuesta y notificaciones
        let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(id_agencia).await {
            Some(agencia.nombre.clone())
        } else {
            None
        };
        
        let estado_texto = if deuda_saldada { "completo" } else { "parcial" };
        let titulo_notif = if deuda_saldada {
            "Pago Completo Registrado"
        } else {
            "Pago Parcial Registrado"
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            titulo_notif,
            &format!(
                "Se ha registrado un pago {} para el file #{}.\nAgencia: {}\nMonto pagado ahora: S/ {}\nMonto total deuda: S/ {}\nNuevo pendiente: S/ {}",
                estado_texto,
                id_file,
                agencia_nombre.clone().unwrap_or_else(|| "Desconocida".to_string()),
                request.monto,
                monto_total_file,
                &monto_total_file - &nuevo_total_pagado
            ),
            if deuda_saldada { NotificationType::Success } else { NotificationType::Info },
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await {
            warn!("Error al enviar notificacion de pago registrado: {}", e);
        }
        
        Ok(self.pago_file_to_response(pago_registrado, agencia_nombre).await)
    }

    /// Verificar pago de file (admin verifica)
    #[instrument(skip(self))]
    pub async fn verificar_pago_file(
        &self,
        request: VerificarPagoFileRequest,
        verificado_por: i32,
    ) -> Result<PagoFileResponse, ApplicationError> {
        // Obtener pago actual
        let _pago = self.pago_file_repository
            .find_by_id(request.id_pago_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", request.id_pago_file)))?;
        
        let estado = if request.aprobado { "verificado" } else { "rechazado" };
        
        let update = UpdatePagoFileModel {
            estado: Some(estado),
            verificado_por: Some(verificado_por),
            verificado_at: Some(Utc::now()),
            notas: request.notas.as_deref(),
            ..Default::default()
        };
        
        let pago_actualizado = self.pago_file_repository
            .update(request.id_pago_file, update)
            .await?;
        
        if request.aprobado {
            info!("Pago de file {} verificado por {}", request.id_pago_file, verificado_por);
        } else {
            warn!("Pago de file {} rechazado por verificador {}", request.id_pago_file, verificado_por);
        }
        
        let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(pago_actualizado.id_agencia).await {
            Some(agencia.nombre)
        } else {
            None
        };
        
        Ok(self.pago_file_to_response(pago_actualizado, agencia_nombre).await)
    }

    // ========================================================================
    // PAGOS A PROVEEDORES
    // ========================================================================

    /// Lista pagos a proveedores con filtros
    #[instrument(skip(self))]
    pub async fn list_pagos_proveedores(
        &self,
        tipo_proveedor: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PagoProveedorResponse>, i64), ApplicationError> {
        let pagos = self.pago_proveedor_repository
            .find_filtered(tipo_proveedor, estado, fecha_desde, fecha_hasta, limit, offset)
            .await?;
        
        let total = self.pago_proveedor_repository
            .count_filtered(tipo_proveedor, estado, fecha_desde, fecha_hasta)
            .await?;
        
        let mut responses: Vec<PagoProveedorResponse> = Vec::new();
        for p in pagos {
            responses.push(self.pago_proveedor_to_response(p).await);
        }
        
        Ok((responses, total))
    }

    /// Crear pago a proveedor (cuando se asigna un servicio)
    #[instrument(skip(self))]
    pub async fn create_pago_proveedor(
        &self,
        request: CreatePagoProveedorRequest,
        created_by: Option<i32>,
    ) -> Result<PagoProveedorResponse, ApplicationError> {
        let new_pago = NewPagoProveedorModel {
            tipo_proveedor: &request.tipo_proveedor,
            id_transporte: request.id_transporte,
            id_restaurante: request.id_restaurante,
            id_guia: request.id_guia,
            id_file_tour: request.id_file_tour,
            id_file_vehiculo: request.id_file_vehiculo,
            id_file_restaurante: request.id_file_restaurante,
            id_file_guia: request.id_file_guia,
            monto: request.monto,
            estado: "pendiente",
            notas: request.notas.as_deref(),
            created_by,
        };
        
        let pago = self.pago_proveedor_repository
            .create(new_pago)
            .await?;
        
        info!("Pago a proveedor creado: {} - {} ({})", pago.id, pago.tipo_proveedor, pago.monto);
        
        Ok(self.pago_proveedor_to_response(pago).await)
    }

    /// Auto-crear pago a proveedor al asignar un servicio (monto=0, estado=pendiente)
    /// Similar a como pagos_files se auto-crea al confirmar un file.
    /// Verifica que no exista ya un pago_proveedor para la misma relación.
    #[instrument(skip(self))]
    pub async fn auto_create_pago_proveedor(
        &self,
        tipo_proveedor: &str,
        id_transporte: Option<i32>,
        id_restaurante: Option<i32>,
        id_guia: Option<i32>,
        id_file_tour: Option<i32>,
        id_file_vehiculo: Option<i32>,
        id_file_restaurante: Option<i32>,
        id_file_guia: Option<i32>,
        created_by: Option<i32>,
    ) -> Result<PagoProveedorResponse, ApplicationError> {
        // Verificar si ya existe un pago_proveedor para esta relación
        let existing = self.pago_proveedor_repository
            .find_by_file_relation(tipo_proveedor, id_file_vehiculo, id_file_restaurante, id_file_guia)
            .await?;
        
        if let Some(existing) = existing {
            info!("Pago a proveedor ya existe para esta relación: {} (tipo: {})", existing.id, tipo_proveedor);
            return Ok(self.pago_proveedor_to_response(existing).await);
        }
        
        let new_pago = NewPagoProveedorModel {
            tipo_proveedor,
            id_transporte,
            id_restaurante,
            id_guia,
            id_file_tour,
            id_file_vehiculo,
            id_file_restaurante,
            id_file_guia,
            monto: BigDecimal::from(0),
            estado: "pendiente",
            notas: None,
            created_by,
        };
        
        let pago = self.pago_proveedor_repository
            .create(new_pago)
            .await?;
        
        info!("Pago a proveedor auto-creado al asignar servicio: {} - {} (monto=0)", pago.id, pago.tipo_proveedor);
        
        Ok(self.pago_proveedor_to_response(pago).await)
    }

    /// Marcar pago a proveedor como pagado
    #[instrument(skip(self))]
    pub async fn marcar_pago_proveedor_pagado(
        &self,
        id_pago_proveedor: i32,
        request: MarcarPagoProveedorPagadoRequest,
        pagado_by: i32,
    ) -> Result<PagoProveedorResponse, ApplicationError> {
        // Obtener pago actual
        let pago = self.pago_proveedor_repository
            .find_by_id(id_pago_proveedor)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id_pago_proveedor)))?;
        
        if pago.estado == "pagado" {
            return Err(ApplicationError::Validation("El pago ya fue marcado como pagado".to_string()));
        }
        
        let comprobante_url: Option<&str> = request.comprobante_url.as_deref();
        let comprobante_key: Option<&str> = None;
        
        let update = UpdatePagoProveedorModel {
            monto: request.monto.clone(),
            estado: Some("pagado"),
            fecha_pago: Some(Utc::now()),
            comprobante_url,
            comprobante_key,
            notas: request.notas.as_deref(),
            pagado_by: Some(pagado_by),
        };
        
        let pago_actualizado = self.pago_proveedor_repository
            .update(id_pago_proveedor, update)
            .await?;
        
        info!("Pago a proveedor {} marcado como pagado por {}", id_pago_proveedor, pagado_by);
        
        Ok(self.pago_proveedor_to_response(pago_actualizado).await)
    }

    // ========================================================================
    // HELPERS - CONVERSION A RESPONSES
    // ========================================================================

    async fn pago_file_to_response(&self, p: PagoFileModel, agencia_nombre: Option<String>) -> PagoFileResponse {
        self.pago_file_to_response_with_calculated_pending(p, agencia_nombre, None).await
    }

    async fn pago_file_to_response_with_calculated_pending(
        &self,
        p: PagoFileModel,
        agencia_nombre: Option<String>,
        calculated_monto_pendiente: Option<BigDecimal>,
    ) -> PagoFileResponse {
        // Obtener codigo del file
        let file_code = if let Ok(Some(file)) = self.file_repository.find_by_id(p.id_file).await {
            file.file_code
        } else {
            None
        };

        let monto_pendiente = calculated_monto_pendiente.unwrap_or_else(|| &p.monto_total - &p.monto_pagado);

        // Obtener tour_nombre si hay file_tour asociado
        let tour_nombre = if let Some(ft_id) = p.id_file_tour {
            if let Ok(Some(ft)) = self.file_tour_repository.find_by_id(ft_id).await {
                if let Ok(Some(tour)) = self.tour_repository.find_by_id(ft.id_tour).await {
                    Some(tour.nombre)
                } else { None }
            } else { None }
        } else { None };

        use bigdecimal::ToPrimitive;

        PagoFileResponse {
            id: p.id,
            id_file: p.id_file,
            file_code,
            id_agencia: p.id_agencia,
            agencia_nombre,
            monto_total: p.monto_total,
            monto_pagado: p.monto_pagado,
            monto_pendiente,
            estado: p.estado,
            fecha_vencimiento: p.fecha_vencimiento.map(|d| d.to_string()),
            comprobante_url: p.comprobante_url,
            verificado_por: p.verificado_por,
            verificador_nombre: None,
            verificado_at: p.verificado_at,
            notas: p.notas,
            created_at: p.created_at,
            id_file_tour: p.id_file_tour,
            tour_nombre,
            tipo_registro: p.tipo_registro,
            entradas: p.entradas,
            entrada_precio: p.entrada_precio.as_ref().and_then(|v| v.to_f64()),
        }
    }

    async fn pago_proveedor_to_response(&self, p: PagoProveedorModel) -> PagoProveedorResponse {
        let proveedor_id = match p.tipo_proveedor.as_str() {
            "transporte" => p.id_transporte.unwrap_or(0),
            "restaurante" => p.id_restaurante.unwrap_or(0),
            "guia" => p.id_guia.unwrap_or(0),
            _ => 0,
        };

        // Obtener nombre del proveedor
        let proveedor_nombre = match p.tipo_proveedor.as_str() {
            "transporte" => {
                if let Some(id) = p.id_transporte {
                    self.transporte_repository.find_by_id(id).await.ok().flatten().map(|t| t.nombre)
                } else { None }
            },
            "restaurante" => {
                if let Some(id) = p.id_restaurante {
                    self.restaurante_repository.find_by_id(id).await.ok().flatten().map(|r| r.nombre)
                } else { None }
            },
            "guia" => {
                if let Some(id) = p.id_guia {
                    if let Ok(Some(guia)) = self.guia_repository.find_by_id(id).await {
                        if let Ok(Some(persona)) = self.persona_repository.find_by_id(guia.id_persona).await {
                            Some(format!("{} {}", persona.nombre, persona.apellidos))
                        } else {
                            Some(format!("Guía #{}", id))
                        }
                    } else { None }
                } else { None }
            },
            _ => None,
        };

        // Obtener info del file_tour (file_code, tour_nombre, fecha_tour)
        let mut file_code = None;
        let mut tour_nombre = None;
        let mut fecha_tour = None;
        if let Some(ft_id) = p.id_file_tour {
            if let Ok(Some(ft)) = self.file_tour_repository.find_by_id(ft_id).await {
                fecha_tour = ft.fecha_tour.map(|d| d.to_string());
                // file_code from parent file
                if let Ok(Some(file)) = self.file_repository.find_by_id(ft.id_file).await {
                    file_code = file.file_code;
                }
                // tour nombre
                if let Ok(Some(tour)) = self.tour_repository.find_by_id(ft.id_tour).await {
                    tour_nombre = Some(tour.nombre);
                }
            }
        }

        // Obtener nombre del usuario que pagó
        let pagado_por = if let Some(user_id) = p.pagado_by {
            self.user_repository.find_by_id(user_id).await.ok().flatten().map(|u| u.username)
        } else { None };

        PagoProveedorResponse {
            id: p.id,
            tipo_proveedor: p.tipo_proveedor,
            proveedor_id,
            proveedor_nombre,
            id_file_tour: p.id_file_tour,
            id_file_vehiculo: p.id_file_vehiculo,
            id_file_restaurante: p.id_file_restaurante,
            id_file_guia: p.id_file_guia,
            file_code,
            tour_nombre,
            fecha_tour,
            monto: p.monto,
            estado: p.estado,
            fecha_pago: p.fecha_pago,
            comprobante_url: p.comprobante_url,
            notas: p.notas,
            created_at: p.created_at,
            pagado_por,
        }
    }
}
