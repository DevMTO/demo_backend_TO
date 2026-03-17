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
use crate::application::dtos::AuditInfo;
use crate::application::services::chat_service::ChatService;
use crate::application::ports::{
    PagoFileRepositoryPort, PagoProveedorRepositoryPort,
    AgenciaRepositoryPort, HotelRepositoryPort, FileRepositoryPort, NotificationServicePort,
    FileTourRepositoryPort, TourRepositoryPort,
    TransporteRepositoryPort, RestauranteRepositoryPort, GuiaRepositoryPort,
    UserRepositoryPort, PersonaRepositoryPort, EntradaRepositoryPort,
    CadenaHoteleraRepositoryPort,
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
    hotel_repository: Arc<dyn HotelRepositoryPort>,
    file_repository: Arc<dyn FileRepositoryPort>,
    notification_service: Arc<dyn NotificationServicePort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    tour_repository: Arc<dyn TourRepositoryPort>,
    transporte_repository: Arc<dyn TransporteRepositoryPort>,
    restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
    guia_repository: Arc<dyn GuiaRepositoryPort>,
    user_repository: Arc<dyn UserRepositoryPort>,
    persona_repository: Arc<dyn PersonaRepositoryPort>,
    entrada_repository: Arc<dyn EntradaRepositoryPort>,
    cadena_hotelera_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
    chat_service: Arc<ChatService>,
}

impl ContabilidadService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        hotel_repository: Arc<dyn HotelRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
        notification_service: Arc<dyn NotificationServicePort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
        tour_repository: Arc<dyn TourRepositoryPort>,
        transporte_repository: Arc<dyn TransporteRepositoryPort>,
        restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
        guia_repository: Arc<dyn GuiaRepositoryPort>,
        user_repository: Arc<dyn UserRepositoryPort>,
        persona_repository: Arc<dyn PersonaRepositoryPort>,
        entrada_repository: Arc<dyn EntradaRepositoryPort>,
        cadena_hotelera_repository: Arc<dyn CadenaHoteleraRepositoryPort>,
        chat_service: Arc<ChatService>,
    ) -> Self {
        Self {
            pago_file_repository,
            pago_proveedor_repository,
            agencia_repository,
            hotel_repository,
            file_repository,
            notification_service,
            file_tour_repository,
            tour_repository,
            transporte_repository,
            restaurante_repository,
            guia_repository,
            user_repository,
            persona_repository,
            entrada_repository,
            cadena_hotelera_repository,
            chat_service,
        }
    }

    // ========================================================================
    // DASHBOARDS
    // ========================================================================

    /// Obtiene el dashboard de contabilidad para una agencia
    #[instrument(skip(self))]
    pub async fn get_agencia_dashboard(&self, id_entidad: i32, entidad: Option<&str>) -> Result<AgenciaContabilidadDashboard, ApplicationError> {
        // Obtener nombre y política de pago según tipo de entidad
        let (nombre_entidad, pago_anticipado_val, tipo_vencimiento_val) = if entidad == Some("cadenas_hoteleras") {
            let cadena = self.cadena_hotelera_repository
                .find_by_id(id_entidad)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Cadena hotelera {} no encontrada", id_entidad)))?;
            (cadena.nombre, false, Some("mensual".to_string()))
        } else if entidad == Some("hoteles") {
            let hotel = self.hotel_repository
                .find_by_id(id_entidad)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Hotel {} no encontrado", id_entidad)))?;
            (hotel.nombre, false, Some("mensual".to_string()))
        } else {
            let agencia = self.agencia_repository
                .find_by_id(id_entidad)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Agencia {} no encontrada", id_entidad)))?;
            (agencia.nombre.clone(), agencia.pago_anticipado, agencia.tipo_vencimiento)
        };
        
        // Obtener todos los pagos de files de esta agencia
        let all_pagos = self.pago_file_repository
            .find_by_entidad(id_entidad, entidad, 1000, 0)
            .await?;
        
        // Separar rechazados antes de agrupar (evita doble conteo con el nuevo pendiente)
        let (rechazados, activos): (Vec<_>, Vec<_>) = all_pagos.into_iter()
            .filter(|p| p.estado != "cancelado" && p.estado != "no_show"
                && (p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final"))
            .partition(|p| p.estado == "rechazado");

        let pagos = activos;
        
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
        
        for file_pagos in file_groups.values() {
            // monto_total: suma de TODAS las deudas por tour
            let monto_total = file_pagos.iter()
                .filter(|p| p.tipo_registro == "deuda")
                .map(|p| &p.monto_total)
                .fold(zero.clone(), |acc, m| acc + m);
            
            // monto_pagado: suma de todos los montos pagados (deudas + pagos)
            let monto_pagado_file = file_pagos.iter()
                .filter(|p| p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final")
                .map(|p| &p.monto_pagado)
                .fold(zero.clone(), |acc, m| acc + m);
            
            let monto_pendiente_file = &monto_total - &monto_pagado_file;
            
            global_monto_total += &monto_total;
            global_monto_pagado += &monto_pagado_file;
            
            // Usar un registro deuda como base (puede tener pagado > 0 tras pagos)
            let base = file_pagos.iter()
                .find(|p| p.tipo_registro == "deuda")
                .cloned()
                .unwrap_or_else(|| file_pagos[0].clone());
            
            // Determinar estado global del file
            let tolerancia = BigDecimal::from_str("0.01").unwrap();
            let is_fully_paid = monto_pendiente_file <= tolerancia;
            let has_partial = monto_pagado_file > zero;
            let all_verified = file_pagos.iter().all(|p| p.estado == "verificado");
            let has_pending_verification = file_pagos.iter().any(|p| p.estado == "pendiente_verificacion");

            let overall_estado = if is_fully_paid && all_verified {
                "verificado"
            } else if has_pending_verification {
                "pendiente_verificacion"
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
                Some(nombre_entidad.clone()),
                Some(monto_pendiente_file.clone()),
            ).await;
            
            if overall_estado == "pendiente" || overall_estado == "parcial" || overall_estado == "vencido" || overall_estado == "pendiente_verificacion" {
                files_pendientes.push(response);
            } else {
                if ultimos_pagos.len() < 10 {
                    ultimos_pagos.push(response);
                }
            }
        }
        
        let monto_pendiente = &global_monto_total - &global_monto_pagado;

        // Construir respuestas individuales para rechazados
        let mut files_rechazados: Vec<PagoFileResponse> = Vec::new();
        for r in rechazados {
            let response = self.pago_file_to_response(r, Some(nombre_entidad.clone())).await;
            files_rechazados.push(response);
        }

        Ok(AgenciaContabilidadDashboard {
            id_entidad,
            nombre_agencia: nombre_entidad,
            total_files: file_groups.len() as i32,
            monto_total_files: global_monto_total,
            monto_pagado: global_monto_pagado,
            monto_pendiente,
            pago_anticipado: pago_anticipado_val,
            tipo_vencimiento: tipo_vencimiento_val,
            files_pendientes,
            ultimos_pagos,
            files_rechazados,
        })
    }

    // ========================================================================
    // PAGOS DE FILES
    // ========================================================================

    /// Obtener un pago_file por ID (para validaciones de ownership)
    pub async fn get_pago_file_by_id(&self, id: i32) -> Result<PagoFileModel, ApplicationError> {
        self.pago_file_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id)))
    }

    /// Lista pagos de files con filtros
    #[instrument(skip(self))]
    pub async fn list_pagos_files(
        &self,
        id_entidad: Option<i32>,
        entidad: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PagoFileResponse>, i64), ApplicationError> {
        let pagos = self.pago_file_repository
            .find_filtered(id_entidad, entidad, estado, fecha_desde, fecha_hasta, limit, offset)
            .await?;

        let total = self.pago_file_repository
            .count_filtered(id_entidad, entidad, estado, fecha_desde, fecha_hasta)
            .await?;

        // Group pagos by file to calculate cumulative monto_pagado per file
        let mut file_ids: Vec<i32> = pagos.iter().map(|p| p.id_file).collect();
        file_ids.sort_unstable();
        file_ids.dedup();

        // Para cada file, calcular totales por file_tour (para monto_pendiente acumulativo)
        // clave: (id_file, id_file_tour) → (monto_total_tour, monto_pagado_acum_tour)
        let mut tour_pagos_totals: std::collections::HashMap<(i32, Option<i32>), (BigDecimal, BigDecimal)> = std::collections::HashMap::new();
        
        for id_file in file_ids {
            if let Ok(all_pagos) = self.pago_file_repository.find_all_by_file(id_file).await {
                // Calcular pendiente por file_tour (para deudas)
                for deuda in all_pagos.iter().filter(|p| p.tipo_registro == "deuda") {
                    let pagado_en_deuda = deuda.monto_pagado.clone();
                    let pagado_en_pagos: BigDecimal = all_pagos.iter()
                        .filter(|p| (p.tipo_registro == "pago" || p.tipo_registro == "pago_final") && p.id_file_tour == deuda.id_file_tour)
                        .map(|p| &p.monto_pagado)
                        .fold(BigDecimal::from_str("0").unwrap(), |acc, m| acc + m);
                    let acumulado = &pagado_en_deuda + &pagado_en_pagos;
                    tour_pagos_totals.insert(
                        (id_file, deuda.id_file_tour),
                        (deuda.monto_total.clone(), acumulado),
                    );
                }
            }
        }

        let mut responses = Vec::new();
        for p in pagos {
            let agencia_nombre = if p.entidad.as_deref() == Some("hoteles") {
                if let Ok(Some(hotel)) = self.hotel_repository.find_by_id(p.id_entidad).await {
                    Some(hotel.nombre)
                } else {
                    None
                }
            } else if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(p.id_entidad).await {
                Some(agencia.nombre)
            } else {
                None
            };

            // Calcular pendiente según tipo de registro
            let monto_pendiente = if p.tipo_registro == "deuda" {
                // Para deudas: pendiente = monto_total_tour - monto_pagado_acum_tour
                let (total_tour, pagado_tour) = tour_pagos_totals
                    .get(&(p.id_file, p.id_file_tour))
                    .cloned()
                    .unwrap_or_else(|| (p.monto_total.clone(), p.monto_pagado.clone()));
                &total_tour - &pagado_tour
            } else {
                // Para pagos, cancelaciones, etc: el pendiente no aplica directamente
                BigDecimal::from_str("0").unwrap()
            };

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
        let id_entidad = pago_original.id_entidad;

        // Guard: no permitir pagos en registros cancelados o no_show
        if pago_original.estado == "cancelado" || pago_original.estado == "no_show" {
            return Err(ApplicationError::Validation(
                format!("No se puede registrar pago en un registro con estado '{}'", pago_original.estado)
            ));
        }

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
            .filter(|p| p.tipo_registro == "deuda" || p.tipo_registro == "pago" || p.tipo_registro == "pago_final")
            .map(|p| &p.monto_pagado)
            .fold(zero.clone(), |acc, m| acc + m);
        
        // Calcular la siguiente cuota (max cuota actual + 1)
        let max_cuota: i16 = all_records.iter()
            .filter_map(|p| p.cuota)
            .max()
            .unwrap_or(0);
        let next_cuota = max_cuota + 1;
        
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
            .filter(|p| p.tipo_registro == "deuda" || p.tipo_registro == "pago_final")
            .cloned()
            .collect();
        
        // Calcular cuánto se ha acumulado en pagos por cada deuda (por file_tour)
        let pagos_por_tour: Vec<_> = all_records.iter()
            .filter(|p| p.tipo_registro == "pago" || p.tipo_registro == "pago_final")
            .cloned()
            .collect();
        
        // Para cada deuda, calcular su monto_pagado real (monto de la deuda + pagos asociados a ese tour)
        struct DeudaInfo {
            deuda_id: i32,
            id_file_tour: Option<i32>,
            monto_total: BigDecimal,
            monto_pagado_original: BigDecimal,
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
                monto_pagado_original: pagado_en_deuda,
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
        deudas_con_entradas_pendientes.sort_by(|a, b| a.monto_pagado_acum.cmp(&b.monto_pagado_acum));
        
        for d in &mut deudas_con_entradas_pendientes {
            if monto_restante <= zero {
                break;
            }
            
            let entrada_precio = d.entrada_precio.clone().unwrap_or_else(|| BigDecimal::from_str("0").unwrap());
            let entrada_pendiente = if d.monto_pagado_acum < entrada_precio {
                &entrada_precio - &d.monto_pagado_acum
            } else {
                zero.clone()
            };
            
            if entrada_pendiente > zero {
                let a_pagar = monto_restante.clone().min(entrada_pendiente);
                if a_pagar <= zero {
                    continue;
                }
                
                if d.monto_pagado_original == zero {
                    // First payment on this debt - update existing
                    let nuevo_pagado = &d.monto_pagado_acum + &a_pagar;
                    let _pendiente_tras_pago = &d.monto_total - &nuevo_pagado;
                    let estado = "pendiente_verificacion";
                    
                    let update = UpdatePagoFileModel {
                        monto_pagado: Some(nuevo_pagado.clone()),
                        estado: Some(estado),
                        comprobante_url: comprobante_url.as_deref(),
                        comprobante_key: comprobante_key.as_deref(),
                        cuota: Some(Some(next_cuota)),
                        pagado_por: Some(created_by),
                        pagado_at: Some(Some(Utc::now())),
                        updated_by: Some(created_by),
                        ..Default::default()
                    };
                    let updated = self.pago_file_repository.update(d.deuda_id, update).await?;
                    ultimo_pago_id = Some(updated.id);
                } else {
                    // Subsequent payments - create new "pago" entry
                    let pending_after = &d.monto_pendiente - &a_pagar;
                    let estado = "pendiente_verificacion";
                    let tipo_registro = if pending_after <= tolerancia { "pago_final" } else { "pago" };
                    let new_pago = NewPagoFileModel {
                        id_file,
                        id_entidad,
                        entidad: pago_original.entidad.as_deref(),
                        monto_total: d.monto_total.clone(),
                        monto_pagado: a_pagar.clone(),
                        estado,
                        fecha_vencimiento: None,
                        notas: request.notas.as_deref(),
                        created_by,
                        id_file_tour: d.id_file_tour,
                        tipo_registro: tipo_registro,
                        monto_saldo_favor: None,
                        saldo_autorizado: false,
                        saldo_autorizado_por: None,
                        saldo_autorizado_at: None,
                        entradas: false,
                        entrada_precio: None,
                        cuota: Some(next_cuota),
                        pagado_por: created_by,
                        pagado_at: Some(Utc::now()),
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
        }
        
        // FASE 2: Distribuir sobrante en deudas sin entradas (primero las de monto_pagado=0)
        let mut deudas_sin_entradas: Vec<&mut DeudaInfo> = deuda_infos.iter_mut()
            .filter(|d| !d.entradas && d.monto_pendiente > zero)
            .collect();
        deudas_sin_entradas.sort_by(|a, b| a.monto_pagado_acum.cmp(&b.monto_pagado_acum));
        
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
            
            if d.monto_pagado_original == zero {
                // First payment on this debt - update existing (add to existing amount from this payment)
                let nuevo_pagado = &d.monto_pagado_acum + &a_pagar;
                let _pendiente_tras_pago = &d.monto_total - &nuevo_pagado;
                let estado = "pendiente_verificacion";
                
                let update = UpdatePagoFileModel {
                    monto_pagado: Some(nuevo_pagado),
                    estado: Some(estado),
                    comprobante_url: comprobante_url.as_deref(),
                    comprobante_key: comprobante_key.as_deref(),
                    cuota: Some(Some(next_cuota)),
                    pagado_por: Some(created_by),
                    pagado_at: Some(Some(Utc::now())),
                    updated_by: Some(created_by),
                    ..Default::default()
                };
                let updated = self.pago_file_repository.update(d.deuda_id, update).await?;
                ultimo_pago_id = Some(updated.id);
            } else {
                // Subsequent payments - create new "pago" entry
                let pending_after = &d.monto_pendiente - &a_pagar;
                let estado = if pending_after <= tolerancia { "pagado" } else { "parcial" };
                let tipo_registro = if pending_after <= tolerancia { "pago_final" } else { "pago" };
                let new_pago = NewPagoFileModel {
                    id_file,
                    id_entidad,
                    entidad: pago_original.entidad.as_deref(),
                    monto_total: d.monto_total.clone(),
                    monto_pagado: a_pagar.clone(),
                    estado,
                    fecha_vencimiento: None,
                    notas: request.notas.as_deref(),
                    created_by,
                    id_file_tour: d.id_file_tour,
                    tipo_registro: tipo_registro,
                    monto_saldo_favor: None,
                    saldo_autorizado: false,
                    saldo_autorizado_por: None,
                    saldo_autorizado_at: None,
                    entradas: false,
                    entrada_precio: None,
                    cuota: Some(next_cuota),
                    pagado_por: created_by,
                    pagado_at: Some(Utc::now()),
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

        let monto_pendiente = &monto_total_file - &monto_pagado_global;
        let deuda_saldada = request.monto >= (&monto_pendiente - &tolerancia);
        
        info!("Pago distribuido para file {}: S/ {} (pendiente actual: S/ {})", 
            id_file, request.monto, monto_pendiente);

        // 6. Preparar respuesta y notificaciones
        let agencia_nombre = self.resolve_entity_name(id_entidad, pago_original.entidad.as_deref()).await;
        
        let estado_texto = if deuda_saldada { "completo" } else { "parcial" };
        let titulo_notif = if deuda_saldada {
            "Pago Completo - Pendiente Verificación"
        } else {
            "Pago Parcial - Pendiente Verificación"
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            titulo_notif,
            &format!(
                "Se ha registrado un pago {} para el file #{}.\nAgencia: {}\nMonto pagado ahora: S/ {}\nMonto total deuda: S/ {}\nPendiente actual: S/ {}\n\nEl pago está pendiente de verificación.",
                estado_texto,
                id_file,
                agencia_nombre.clone().unwrap_or_else(|| "Desconocida".to_string()),
                request.monto,
                monto_total_file,
                monto_pendiente
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
        user_info: AuditInfo,
    ) -> Result<PagoFileResponse, ApplicationError> {
        let verificado_por = user_info.user_id;
        
        // Obtener pago actual
        let pago = self.pago_file_repository
            .find_by_id(request.id_pago_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", request.id_pago_file)))?;

        let tolerancia = BigDecimal::from_str("0.01").unwrap();
        let pendiente = &pago.monto_total - &pago.monto_pagado;
        
        let estado = if request.aprobado {
            if pendiente <= tolerancia { "pagado" } else { "parcial" }
        } else {
            "rechazado"
        };
        
        let update = UpdatePagoFileModel {
            estado: Some(estado),
            verificado_por: Some(verificado_por),
            verificado_at: Some(Utc::now()),
            ..Default::default()
        };
        
        let pago_actualizado = self.pago_file_repository
            .update(request.id_pago_file, update)
            .await?;
        
        let agencia_nombre = self.resolve_entity_name(pago_actualizado.id_entidad, pago_actualizado.entidad.as_deref()).await;
        
        if request.aprobado {
            info!("Pago de file {} aprobado por {} - estado: {}", request.id_pago_file, verificado_por, estado);
            
            // Actualizar monto_pagado en la tabla files
            let id_file = pago_actualizado.id_file;
            let cuota_verificada = pago_actualizado.cuota;
            let all_pagos = self.pago_file_repository.find_all_by_file(id_file).await?;
            let zero = BigDecimal::from_str("0").unwrap();
            let monto_cuota: BigDecimal = all_pagos.iter()
                .filter(|p| p.cuota == cuota_verificada)
                .map(|p| &p.monto_pagado)
                .fold(zero.clone(), |acc, m| acc + m);
            
            let file = self.file_repository
                .find_by_id(id_file)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", id_file)))?;
            let monto_pagado_anterior = file.monto_pagado.clone();
            let monto_pagado_nuevo = &monto_pagado_anterior + &monto_cuota;
            let mut file_update = file.clone();
            file_update.monto_pagado = monto_pagado_nuevo.clone();
            self.file_repository.update(&file_update).await?;
            info!("File {} monto_pagado actualizado de S/ {} a S/ {} tras verificación de cuota {:?} (monto cuota: S/ {})", 
                id_file, monto_pagado_anterior, monto_pagado_nuevo, cuota_verificada, monto_cuota);
            
            // Notificar a la agencia que su pago fue aprobado
            let (roles_notif, titulo) = if estado == "pagado" {
                (vec![UserRole::Agencias, UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::Hoteles, UserRole::HotelesGerente], "Pago aprobado")
            } else {
                (vec![UserRole::Agencias, UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::Hoteles, UserRole::HotelesGerente], "Pago parcial aprobado")
            };
            
            if let Err(e) = self.notification_service.notify_roles_for_entity(
                roles_notif,
                pago_actualizado.id_entidad,
                titulo,
                &format!(
                    "El pago para el file #{} ha sido aprobado por el administrador.\nMonto: S/ {}\nEstado: {}",
                    pago_actualizado.id_file,
                    pago.monto_pagado,
                    if estado == "pagado" { "Pagado" } else { "Parcial" }
                ),
                NotificationType::Success,
                NotificationCategory::Financial,
                NotificationPriority::High,
                Some(verificado_por),
            ).await {
                warn!("Error al notificar a la agencia del pago aprobado: {}", e);
            }
        } else {
            warn!("Pago de file {} rechazado por verificador {}", request.id_pago_file, verificado_por);
            
            // Notificar a la agencia que su pago fue rechazado
            if let Err(e) = self.notification_service.notify_roles_for_entity(
                vec![UserRole::Agencias, UserRole::AgenciasContador, UserRole::AgenciasGerente, UserRole::Hoteles, UserRole::HotelesGerente],
                pago_actualizado.id_entidad,
                "Pago rechazado",
                &format!(
                    "El pago para el file #{} ha sido rechazado por el administrador.\nMonto: S/ {}\nMotivo: {}",
                    pago_actualizado.id_file,
                    pago.monto_pagado,
                    request.notas.as_deref().unwrap_or("Sin motivo especificado")
                ),
                NotificationType::Error,
                NotificationCategory::Financial,
                NotificationPriority::High,
                Some(verificado_por),
            ).await {
                warn!("Error al notificar a la agencia del pago rechazado: {}", e);
            }

            // Duplicar la fila como pendiente para que la entidad vuelva a pagar
            let nuevo_pago = NewPagoFileModel {
                id_file: pago.id_file,
                id_entidad: pago.id_entidad,
                entidad: pago.entidad.as_deref(),
                monto_total: pago.monto_total.clone(),
                monto_pagado: BigDecimal::from(0),
                estado: "pendiente",
                fecha_vencimiento: pago.fecha_vencimiento,
                notas: None,
                created_by: Some(verificado_por),
                id_file_tour: pago.id_file_tour,
                tipo_registro: &pago.tipo_registro,
                monto_saldo_favor: None,
                saldo_autorizado: false,
                saldo_autorizado_por: None,
                saldo_autorizado_at: None,
                entradas: pago.entradas,
                entrada_precio: pago.entrada_precio.clone(),
                cuota: Some(0),
                pagado_por: None,
                pagado_at: None,
            };
            self.pago_file_repository.create(nuevo_pago).await?;
        }
        
        if let Some(nota) = request.notas {
            let _ = self.chat_service.chat_file(
                pago_actualizado.id_file,
                Some(nota),
                Some(user_info),
            ).await;
        }
        
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
            monto_total: request.monto,
            estado: "pendiente",
            notas: request.notas.as_deref(),
            created_by,
            id_entrada: request.id_entrada,
            id_file_entrada: request.id_file_entrada,
        };

        let pago = self.pago_proveedor_repository
            .create(new_pago)
            .await?;

        info!("Pago a proveedor creado: {} - {} ({})", pago.id, pago.tipo_proveedor, pago.monto_total);
        
        Ok(self.pago_proveedor_to_response(pago).await)
    }

    /// Auto-crear pago a proveedor al asignar un servicio (estado=pendiente)
    /// Similar a como pagos_files se auto-crea al confirmar un file.
    /// Verifica que no exista ya un pago_proveedor para la misma relación.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self))]
    pub async fn auto_create_pago_proveedor(
        &self,
        tipo_proveedor: &str,
        id_transporte: Option<i32>,
        id_restaurante: Option<i32>,
        id_guia: Option<i32>,
        id_entrada: Option<i32>,
        id_file_tour: Option<i32>,
        id_file_vehiculo: Option<i32>,
        id_file_restaurante: Option<i32>,
        id_file_guia: Option<i32>,
        id_file_entrada: Option<i32>,
        monto_total: Option<BigDecimal>,
        created_by: Option<i32>,
    ) -> Result<PagoProveedorResponse, ApplicationError> {
        // Verificar si ya existe un pago_proveedor para esta relación
        let existing = self.pago_proveedor_repository
            .find_by_file_relation(tipo_proveedor, id_file_vehiculo, id_file_restaurante, id_file_guia, id_file_entrada)
            .await?;

        if let Some(existing) = existing {
            info!("Pago a proveedor ya existe para esta relación: {} (tipo: {})", existing.id, tipo_proveedor);
            return Ok(self.pago_proveedor_to_response(existing).await);
        }

        let monto = monto_total.unwrap_or_else(|| BigDecimal::from(0));

        let new_pago = NewPagoProveedorModel {
            tipo_proveedor,
            id_transporte,
            id_restaurante,
            id_guia,
            id_file_tour,
            id_file_vehiculo,
            id_file_restaurante,
            id_file_guia,
            monto_total: monto.clone(),
            estado: "pendiente",
            notas: None,
            created_by,
            id_entrada,
            id_file_entrada,
        };

        let pago = self.pago_proveedor_repository
            .create(new_pago)
            .await?;

        info!("Pago a proveedor auto-creado al asignar servicio: {} - {} (monto={})", pago.id, pago.tipo_proveedor, monto);

        Ok(self.pago_proveedor_to_response(pago).await)
    }

    /// Marcar pago a proveedor como pagado
    /// Si el monto es mayor al monto_pagado original, distribuye el excedente a otros pagos
    /// del mismo tipo_proveedor, tour_id y fecha_tour
    #[instrument(skip(self))]
    pub async fn marcar_pago_proveedor_pagado(
        &self,
        id_pago_proveedor: i32,
        request: MarcarPagoProveedorPagadoRequest,
        pagado_by: i32,
    ) -> Result<PagoProveedorResponse, ApplicationError> {
        let pago = self.pago_proveedor_repository
            .find_by_id(id_pago_proveedor)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", id_pago_proveedor)))?;

        if pago.estado == "pagado" {
            return Err(ApplicationError::Validation("El pago ya fue marcado como pagado".to_string()));
        }

        let monto_pagado_final = request.monto_pagado.clone()
            .or(request.monto.clone())
            .unwrap_or(pago.monto_total.clone());

        let update = UpdatePagoProveedorModel {
            monto_total: request.monto.clone(),
            estado: Some("pagado"),
            fecha_pago: Some(Utc::now()),
            comprobante_url: request.comprobante_url.as_deref(),
            comprobante_key: None,
            notas: request.notas.as_deref(),
            pagado_by: Some(pagado_by),
            monto_pagado: Some(monto_pagado_final),
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

    /// Resolve entity name (agencia or hotel) by id_entidad and entidad type
    async fn resolve_entity_name(&self, id_entidad: i32, entidad: Option<&str>) -> Option<String> {
        if entidad == Some("hoteles") {
            if let Ok(Some(hotel)) = self.hotel_repository.find_by_id(id_entidad).await {
                return Some(hotel.nombre);
            }
        } else if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(id_entidad).await {
            return Some(agencia.nombre);
        }
        None
    }

    /// Resuelve user_id → User → Persona → nombre completo
    async fn resolve_user_full_name(&self, user_id: Option<i32>) -> Option<String> {
        let uid = user_id?;
        let user = self.user_repository.find_by_id(uid).await.ok()??;
        let persona_id = user.id_persona?;
        let persona = self.persona_repository.find_by_id(persona_id).await.ok()??;
        Some(format!("{} {}", persona.nombre, persona.apellidos))
    }

    async fn pago_file_to_response(&self, p: PagoFileModel, agencia_nombre: Option<String>) -> PagoFileResponse {
        self.pago_file_to_response_with_calculated_pending(p, agencia_nombre, None).await
    }

    async fn pago_file_to_response_with_calculated_pending(
        &self,
        p: PagoFileModel,
        agencia_nombre: Option<String>,
        calculated_monto_pendiente: Option<BigDecimal>,
    ) -> PagoFileResponse {
        // Obtener datos del file (codigo, fecha_inicio, fecha_fin)
        let file_data = self.file_repository.find_by_id(p.id_file).await.ok().flatten();
        let file_code = file_data.as_ref().and_then(|f| f.file_code.clone());
        let fecha_inicio = file_data.as_ref().map(|f| f.fecha_inicio.to_string());
        let fecha_fin = file_data.as_ref().map(|f| f.fecha_fin.to_string());

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

        // Resolver nombres completos de auditoria
        let verificador_nombre = self.resolve_user_full_name(p.verificado_por).await;
        let created_by_nombre = self.resolve_user_full_name(p.created_by).await;
        let saldo_autorizado_por_nombre = self.resolve_user_full_name(p.saldo_autorizado_por).await;
        let pagado_por_nombre = self.resolve_user_full_name(p.pagado_por).await;
        let updated_by_nombre = self.resolve_user_full_name(p.updated_by).await;

        PagoFileResponse {
            id: p.id,
            id_file: p.id_file,
            file_code,
            id_entidad: p.id_entidad,
            agencia_nombre,
            monto_total: p.monto_total,
            monto_pagado: p.monto_pagado,
            monto_pendiente,
            estado: p.estado,
            fecha_vencimiento: p.fecha_vencimiento.map(|d| d.to_string()),
            comprobante_url: p.comprobante_url,
            verificado_por: p.verificado_por,
            verificador_nombre,
            verificado_at: p.verificado_at,
            notas: p.notas,
            created_at: p.created_at,
            created_by: p.created_by,
            created_by_nombre,
            saldo_autorizado_por: p.saldo_autorizado_por,
            saldo_autorizado_por_nombre,
            pagado_por: p.pagado_por,
            pagado_por_nombre,
            pagado_at: p.pagado_at,
            updated_by: p.updated_by,
            updated_by_nombre,
            id_file_tour: p.id_file_tour,
            tour_nombre,
            tipo_registro: p.tipo_registro,
            entradas: p.entradas,
            entrada_precio: p.entrada_precio.as_ref().and_then(|v| v.to_f64()),
            cuota: p.cuota,
            entidad: p.entidad,
            fecha_inicio,
            fecha_fin,
        }
    }

    async fn pago_proveedor_to_response(&self, p: PagoProveedorModel) -> PagoProveedorResponse {
        let proveedor_id = match p.tipo_proveedor.as_str() {
            "transporte" => p.id_transporte.unwrap_or(0),
            "restaurante" => p.id_restaurante.unwrap_or(0),
            "guia" => p.id_guia.unwrap_or(0),
            "entrada" => p.id_entrada.unwrap_or(0),
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
            "entrada" => {
                if let Some(id) = p.id_entrada {
                    self.entrada_repository.find_by_id(id).await.ok().flatten().map(|e| e.nombre)
                } else { None }
            },
            _ => None,
        };

        // Obtener info del file_tour (file_code, tour_nombre, fecha_tour, turno_tour)
        let mut file_code = None;
        let mut tour_nombre = None;
        let mut tour_id = None;
        let mut turno_tour = None;
        let mut fecha_tour = None;
        if let Some(ft_id) = p.id_file_tour {
            if let Ok(Some(ft)) = self.file_tour_repository.find_by_id(ft_id).await {
                fecha_tour = ft.fecha_tour.map(|d| d.to_string());
                turno_tour = ft.turno_tour;
                tour_id = Some(ft.id_tour);
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
            id_file_entrada: p.id_file_entrada,
            file_code,
            tour_nombre,
            turno_tour,
            tour_id,
            fecha_tour,
            monto: p.monto_total,
            monto_pagado: p.monto_pagado,
            estado: p.estado,
            fecha_pago: p.fecha_pago,
            comprobante_url: p.comprobante_url,
            notas: p.notas,
            created_at: p.created_at,
            pagado_por,
        }
    }
}
