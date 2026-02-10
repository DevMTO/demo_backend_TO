//! Servicios de contabilidad
//!
//! Incluye:
//! - ContabilidadService: Servicio principal que orquesta operaciones de contabilidad
//! - Dashboard para admin y agencias
//! - Gestión de movimientos, pagos de files y pagos a proveedores

use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use std::str::FromStr;
use tracing::{info, instrument, warn};

use crate::application::dtos::contabilidad_dto::{
    AdminContabilidadDashboard, AgenciaContabilidadDashboard,
    MovimientoResponse, PagoFileResponse, PagoProveedorResponse,
    CreateMovimientoRequest, RegistrarPagoFileRequest, VerificarPagoFileRequest,
    CreatePagoProveedorRequest, MarcarPagoProveedorPagadoRequest,
    TarifaServicioResponse, CreateTarifaServicioRequest, UpdateTarifaServicioRequest,
};
use crate::application::ports::{
    CuentaRepositoryPort, MovimientoRepositoryPort, PagoFileRepositoryPort,
    PagoProveedorRepositoryPort, TarifaServicioRepositoryPort,
    AgenciaRepositoryPort, FileRepositoryPort, NotificationServicePort,
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::{
    UserRole, NotificationType, NotificationCategory, NotificationPriority,
};
use crate::infrastructure::persistence::models::{
    NewMovimientoModel, NewPagoProveedorModel, NewTarifaServicioModel,
    UpdatePagoFileModel, UpdatePagoProveedorModel, UpdateTarifaServicioModel,
    MovimientoModel, PagoFileModel, PagoProveedorModel, TarifaServicioModel,
};

// ============================================================================
// CONTABILIDAD SERVICE
// ============================================================================

pub struct ContabilidadService {
    cuenta_repository: Arc<dyn CuentaRepositoryPort>,
    movimiento_repository: Arc<dyn MovimientoRepositoryPort>,
    pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
    pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
    tarifa_repository: Arc<dyn TarifaServicioRepositoryPort>,
    agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    file_repository: Arc<dyn FileRepositoryPort>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl ContabilidadService {
    pub fn new(
        cuenta_repository: Arc<dyn CuentaRepositoryPort>,
        movimiento_repository: Arc<dyn MovimientoRepositoryPort>,
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
        tarifa_repository: Arc<dyn TarifaServicioRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            cuenta_repository,
            movimiento_repository,
            pago_file_repository,
            pago_proveedor_repository,
            tarifa_repository,
            agencia_repository,
            file_repository,
            notification_service,
        }
    }

    // ========================================================================
    // DASHBOARDS
    // ========================================================================

    /// Obtiene el dashboard de contabilidad para el admin/operador
    #[instrument(skip(self))]
    pub async fn get_admin_dashboard(&self) -> Result<AdminContabilidadDashboard, ApplicationError> {
        // Obtener cuenta del admin
        let cuenta_admin = self.cuenta_repository
            .find_admin_account()
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Cuenta del admin no encontrada".to_string()))?;
        
        // Calcular período (último mes)
        let ahora = Utc::now();
        let hace_un_mes = ahora - Duration::days(30);
        
        // Sumar ingresos y egresos del período
        let total_ingresos = self.movimiento_repository
            .sum_ingresos(cuenta_admin.id, hace_un_mes, ahora)
            .await?;
        
        let total_egresos = self.movimiento_repository
            .sum_egresos(cuenta_admin.id, hace_un_mes, ahora)
            .await?;
        
        let balance_periodo = &total_ingresos - &total_egresos;
        
        // Totales pendientes
        let total_pendiente_cobrar = self.pago_file_repository
            .sum_pendiente_cobrar()
            .await?;
        
        let total_pendiente_pagar = self.pago_proveedor_repository
            .sum_pendiente_pagar()
            .await?;
        
        // Conteos
        let files_pendientes_pago = self.pago_file_repository
            .count_pendientes()
            .await? as i32;
        
        let pagos_proveedores_pendientes = self.pago_proveedor_repository
            .count_pendientes()
            .await? as i32;
        
        // Últimos movimientos
        let ultimos_movimientos_raw = self.movimiento_repository
            .find_by_cuenta(cuenta_admin.id, 10, 0)
            .await?;
        
        let ultimos_movimientos = ultimos_movimientos_raw.into_iter()
            .map(|m| self.movimiento_to_response(m, Some(cuenta_admin.nombre.clone())))
            .collect();
        
        Ok(AdminContabilidadDashboard {
            saldo_actual: cuenta_admin.saldo_actual,
            total_ingresos,
            total_egresos,
            balance_periodo,
            total_pendiente_cobrar,
            total_pendiente_pagar,
            files_pendientes_pago,
            pagos_proveedores_pendientes,
            ultimos_movimientos,
        })
    }

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
            .find_by_agencia(id_agencia, 1000, 0) // Traer todos para calcular totales
            .await?;
        
        // Excluir pagos de files cancelados y no_show de los totales
        let pagos: Vec<_> = all_pagos.into_iter()
            .filter(|p| p.estado != "cancelado" && p.estado != "no_show")
            .collect();
        
        // Calcular totales (solo files activos)
        let total_files = pagos.len() as i32;
        let monto_total_files = pagos.iter()
            .map(|p| &p.monto_total)
            .fold(BigDecimal::from_str("0").unwrap(), |acc, m| acc + m);
        let monto_pagado = pagos.iter()
            .map(|p| &p.monto_pagado)
            .fold(BigDecimal::from_str("0").unwrap(), |acc, m| acc + m);
        let monto_pendiente = &monto_total_files - &monto_pagado;
        
        // Separar files pendientes y últimos pagos
        let mut files_pendientes: Vec<PagoFileResponse> = Vec::new();
        let mut ultimos_pagos: Vec<PagoFileResponse> = Vec::new();
        
        for pago in pagos {
            let response = self.pago_file_to_response(pago.clone(), Some(agencia.nombre.clone())).await;
            if pago.estado == "pendiente" || pago.estado == "parcial" || pago.estado == "vencido" {
                files_pendientes.push(response);
            } else {
                if ultimos_pagos.len() < 10 {
                    ultimos_pagos.push(response);
                }
            }
        }
        
        Ok(AgenciaContabilidadDashboard {
            id_agencia,
            nombre_agencia: agencia.nombre,
            total_files,
            monto_total_files,
            monto_pagado,
            monto_pendiente,
            files_pendientes,
            ultimos_pagos,
        })
    }

    // ========================================================================
    // MOVIMIENTOS
    // ========================================================================

    /// Lista movimientos con filtros
    #[instrument(skip(self))]
    pub async fn list_movimientos(
        &self,
        id_cuenta: Option<i32>,
        tipo: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        referencia_tipo: Option<&str>,
        referencia_id: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MovimientoResponse>, i64), ApplicationError> {
        let movimientos = self.movimiento_repository
            .find_filtered(id_cuenta, tipo, fecha_desde, fecha_hasta, referencia_tipo, referencia_id, limit, offset)
            .await?;
        
        let total = self.movimiento_repository
            .count_filtered(id_cuenta, tipo, fecha_desde, fecha_hasta, referencia_tipo, referencia_id)
            .await?;
        
        // Convertir a responses con nombre de cuenta
        let mut responses = Vec::new();
        for m in movimientos {
            let cuenta_nombre = if let Ok(Some(cuenta)) = self.cuenta_repository.find_by_id(m.id_cuenta).await {
                Some(cuenta.nombre)
            } else {
                None
            };
            responses.push(self.movimiento_to_response(m, cuenta_nombre));
        }
        
        Ok((responses, total))
    }

    /// Crear movimiento manual (ajuste)
    /// 
    /// # Arguments
    /// * `request` - Datos del movimiento
    /// * `created_by` - ID del usuario que crea el movimiento
    /// * `comprobante_url` - URL del comprobante ya subido a Tigris (opcional)
    /// * `comprobante_key` - Key del comprobante en Tigris (opcional)
    #[instrument(skip(self))]
    pub async fn create_movimiento(
        &self,
        request: CreateMovimientoRequest,
        created_by: Option<i32>,
        comprobante_url: Option<String>,
        comprobante_key: Option<String>,
    ) -> Result<MovimientoResponse, ApplicationError> {
        // Obtener cuenta y saldo actual
        let cuenta = self.cuenta_repository
            .find_by_id(request.id_cuenta)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Cuenta {} no encontrada", request.id_cuenta)))?;
        
        let saldo_anterior = cuenta.saldo_actual.clone();
        
        // Calcular nuevo saldo
        let saldo_posterior = if request.tipo == "ingreso" {
            &saldo_anterior + &request.monto
        } else {
            &saldo_anterior - &request.monto
        };
        
        // Crear movimiento con comprobante (si viene URL)
        let new_movimiento = NewMovimientoModel {
            id_cuenta: request.id_cuenta,
            tipo: &request.tipo,
            monto: request.monto,
            concepto: &request.concepto,
            referencia_tipo: None,
            referencia_id: None,
            fecha_movimiento: Utc::now(),
            saldo_anterior: saldo_anterior.clone(),
            saldo_posterior: saldo_posterior.clone(),
            notas: request.notas.as_deref(),
            comprobante_url: comprobante_url.as_deref(),
            comprobante_key: comprobante_key.as_deref(),
            created_by,
        };
        
        let movimiento = self.movimiento_repository
            .create(new_movimiento)
            .await?;
        
        // Actualizar saldo de la cuenta
        self.cuenta_repository
            .update_saldo(request.id_cuenta, saldo_posterior)
            .await?;
        
        if comprobante_url.is_some() {
            info!("Movimiento manual creado con comprobante: {} - {}", movimiento.id, movimiento.concepto);
        } else {
            info!("Movimiento manual creado: {} - {}", movimiento.id, movimiento.concepto);
        }
        
        Ok(self.movimiento_to_response(movimiento, Some(cuenta.nombre)))
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
        
        let mut responses = Vec::new();
        for p in pagos {
            let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(p.id_agencia).await {
                Some(agencia.nombre)
            } else {
                None
            };
            responses.push(self.pago_file_to_response(p, agencia_nombre).await);
        }
        
        Ok((responses, total))
    }

    /// Registrar pago de file (agencia sube comprobante)
    #[instrument(skip(self))]
    pub async fn registrar_pago_file(
        &self,
        request: RegistrarPagoFileRequest,
        created_by: Option<i32>,
        comprobante_url: Option<String>,
        comprobante_key: Option<String>,
    ) -> Result<PagoFileResponse, ApplicationError> {
        // Obtener pago actual
        let pago = self.pago_file_repository
            .find_by_id(request.id_pago_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", request.id_pago_file)))?;
        
        // Validar que hay monto pendiente
        let monto_pendiente = &pago.monto_total - &pago.monto_pagado;
        if request.monto > monto_pendiente {
            return Err(ApplicationError::Validation(
                format!("El monto {} excede el pendiente {}", request.monto, monto_pendiente)
            ));
        }
        
        // Calcular nuevo monto pagado y estado
        let nuevo_monto_pagado = &pago.monto_pagado + &request.monto;
        let nuevo_estado = if nuevo_monto_pagado >= pago.monto_total {
            "pagado"
        } else {
            "parcial"
        };
        
        // Actualizar pago con comprobante si existe
        let update = UpdatePagoFileModel {
            monto_pagado: Some(nuevo_monto_pagado.clone()),
            estado: Some(nuevo_estado),
            comprobante_url: comprobante_url.as_deref(),
            comprobante_key: comprobante_key.as_deref(),
            notas: request.notas.as_deref(),
            ..Default::default()
        };
        
        let pago_actualizado = self.pago_file_repository
            .update(request.id_pago_file, update)
            .await?;
        
        info!("Pago de file {} registrado: {} -> {}", request.id_pago_file, pago.estado, nuevo_estado);
        
        // Obtener nombre de agencia para notificación
        let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(pago_actualizado.id_agencia).await {
            Some(agencia.nombre.clone())
        } else {
            None
        };
        
        // Notificar a los admins del pago registrado
        let estado_texto = if nuevo_estado == "pagado" { "completo" } else { "parcial" };
        let titulo_notif = if nuevo_estado == "pagado" {
            "💵 Pago Completo Registrado"
        } else {
            "💰 Pago Parcial Registrado"
        };
        
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            titulo_notif,
            &format!(
                "Se ha registrado un pago {} para el file #{}.\nAgencia: {}\nMonto pagado: S/ {}\nMonto total: S/ {}\nPendiente: S/ {}",
                estado_texto,
                pago.id_file,
                agencia_nombre.clone().unwrap_or_else(|| "Desconocida".to_string()),
                request.monto,
                pago.monto_total,
                &pago.monto_total - &nuevo_monto_pagado
            ),
            if nuevo_estado == "pagado" { NotificationType::Success } else { NotificationType::Info },
            NotificationCategory::Financial,
            NotificationPriority::High,
            created_by,
        ).await {
            warn!("Error al enviar notificación de pago registrado: {}", e);
        }
        
        Ok(self.pago_file_to_response(pago_actualizado, agencia_nombre).await)
    }

    /// Verificar pago de file (admin verifica)
    #[instrument(skip(self))]
    pub async fn verificar_pago_file(
        &self,
        request: VerificarPagoFileRequest,
        verificado_por: i32,
    ) -> Result<PagoFileResponse, ApplicationError> {
        // Obtener pago actual
        let pago = self.pago_file_repository
            .find_by_id(request.id_pago_file)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Pago {} no encontrado", request.id_pago_file)))?;
        
        if !request.aprobado {
            // Si no se aprueba, volver a pendiente
            let update = UpdatePagoFileModel {
                estado: Some("pendiente"),
                notas: request.notas.as_deref(),
                ..Default::default()
            };
            
            let pago_actualizado = self.pago_file_repository
                .update(request.id_pago_file, update)
                .await?;
            
            warn!("Pago de file {} rechazado por verificador {}", request.id_pago_file, verificado_por);
            
            let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(pago_actualizado.id_agencia).await {
                Some(agencia.nombre)
            } else {
                None
            };
            
            return Ok(self.pago_file_to_response(pago_actualizado, agencia_nombre).await);
        }
        
        // Verificar el pago
        let update = UpdatePagoFileModel {
            verificado_por: Some(verificado_por),
            verificado_at: Some(Utc::now()),
            notas: request.notas.as_deref(),
            ..Default::default()
        };
        
        let pago_actualizado = self.pago_file_repository
            .update(request.id_pago_file, update)
            .await?;
        
        // Si está pagado completamente, crear movimiento de ingreso en cuenta admin
        if pago_actualizado.estado == "pagado" {
            if let Ok(Some(cuenta_admin)) = self.cuenta_repository.find_admin_account().await {
                let saldo_anterior = cuenta_admin.saldo_actual.clone();
                let saldo_posterior = &saldo_anterior + &pago_actualizado.monto_total;
                
                let new_movimiento = NewMovimientoModel {
                    id_cuenta: cuenta_admin.id,
                    tipo: "ingreso",
                    monto: pago_actualizado.monto_total.clone(),
                    concepto: &format!("Pago file #{} - Agencia #{}", pago.id_file, pago.id_agencia),
                    referencia_tipo: Some("file"),
                    referencia_id: Some(pago.id_file),
                    fecha_movimiento: Utc::now(),
                    saldo_anterior,
                    saldo_posterior: saldo_posterior.clone(),
                    notas: request.notas.as_deref(),
                    comprobante_url: pago_actualizado.comprobante_url.as_deref(),
                    comprobante_key: pago_actualizado.comprobante_key.as_deref(),
                    created_by: Some(verificado_por),
                };
                
                let _ = self.movimiento_repository.create(new_movimiento).await;
                let _ = self.cuenta_repository.update_saldo(cuenta_admin.id, saldo_posterior).await;
            }
        }
        
        info!("Pago de file {} verificado por {}", request.id_pago_file, verificado_por);
        
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
        
        let responses: Vec<PagoProveedorResponse> = pagos.into_iter()
            .map(|p| self.pago_proveedor_to_response(p))
            .collect();
        
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
        
        Ok(self.pago_proveedor_to_response(pago))
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
        
        // TODO: Subir comprobante a Tigris si viene en base64
        let comprobante_url: Option<&str> = None;
        let comprobante_key: Option<&str> = None;
        
        let update = UpdatePagoProveedorModel {
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
        
        // Crear movimiento de egreso en cuenta admin
        if let Ok(Some(cuenta_admin)) = self.cuenta_repository.find_admin_account().await {
            let saldo_anterior = cuenta_admin.saldo_actual.clone();
            let saldo_posterior = &saldo_anterior - &pago_actualizado.monto;
            
            let new_movimiento = NewMovimientoModel {
                id_cuenta: cuenta_admin.id,
                tipo: "egreso",
                monto: pago_actualizado.monto.clone(),
                concepto: &format!("Pago a {} #{}", pago.tipo_proveedor, id_pago_proveedor),
                referencia_tipo: Some("pago_proveedor"),
                referencia_id: Some(pago_actualizado.id),
                fecha_movimiento: Utc::now(),
                saldo_anterior,
                saldo_posterior: saldo_posterior.clone(),
                notas: None,
                comprobante_url: None,
                comprobante_key: None,
                created_by: Some(pagado_by),
            };
            
            let _ = self.movimiento_repository.create(new_movimiento).await;
            let _ = self.cuenta_repository.update_saldo(cuenta_admin.id, saldo_posterior).await;
        }
        
        info!("Pago a proveedor {} marcado como pagado por {}", id_pago_proveedor, pagado_by);
        
        Ok(self.pago_proveedor_to_response(pago_actualizado))
    }

    // ========================================================================
    // TARIFAS
    // ========================================================================

    /// Lista todas las tarifas
    #[instrument(skip(self))]
    pub async fn list_tarifas(
        &self,
        tipo_servicio: Option<&str>,
        solo_activas: bool,
    ) -> Result<Vec<TarifaServicioResponse>, ApplicationError> {
        let tarifas = if let Some(tipo) = tipo_servicio {
            self.tarifa_repository.find_by_tipo(tipo, solo_activas).await?
        } else {
            self.tarifa_repository.find_all(solo_activas).await?
        };
        
        let responses: Vec<TarifaServicioResponse> = tarifas.into_iter()
            .map(|t| self.tarifa_to_response(t))
            .collect();
        
        Ok(responses)
    }

    /// Crear tarifa
    #[instrument(skip(self))]
    pub async fn create_tarifa(
        &self,
        request: CreateTarifaServicioRequest,
        created_by: Option<i32>,
    ) -> Result<TarifaServicioResponse, ApplicationError> {
        // Parsear fechas
        let vigente_desde = NaiveDate::parse_from_str(&request.vigente_desde, "%Y-%m-%d")
            .map_err(|_| ApplicationError::Validation("Formato de fecha inválido para vigente_desde. Use YYYY-MM-DD".to_string()))?;
        
        let vigente_hasta = if let Some(ref fecha) = request.vigente_hasta {
            Some(NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
                .map_err(|_| ApplicationError::Validation("Formato de fecha inválido para vigente_hasta. Use YYYY-MM-DD".to_string()))?)
        } else {
            None
        };
        
        let new_tarifa = NewTarifaServicioModel {
            tipo_servicio: &request.tipo_servicio,
            id_servicio: request.id_servicio,
            precio_venta: request.precio_venta,
            precio_costo: request.precio_costo,
            vigente_desde,
            vigente_hasta,
            is_active: true,
            created_by,
        };
        
        let tarifa = self.tarifa_repository
            .create(new_tarifa)
            .await?;
        
        info!("Tarifa creada: {} - {} (servicio {})", tarifa.id, tarifa.tipo_servicio, tarifa.id_servicio);
        
        Ok(self.tarifa_to_response(tarifa))
    }

    /// Actualizar tarifa
    #[instrument(skip(self))]
    pub async fn update_tarifa(
        &self,
        id: i32,
        request: UpdateTarifaServicioRequest,
    ) -> Result<TarifaServicioResponse, ApplicationError> {
        // Verificar que la tarifa existe
        let _tarifa_actual = self.tarifa_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Tarifa {} no encontrada", id)))?;
        
        // Parsear vigente_hasta si se proporciona
        let vigente_hasta = if let Some(ref fecha) = request.vigente_hasta {
            Some(NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
                .map_err(|_| ApplicationError::Validation("Formato de fecha inválido para vigente_hasta. Use YYYY-MM-DD".to_string()))?)
        } else {
            None
        };
        
        let update = UpdateTarifaServicioModel {
            precio_venta: request.precio_venta,
            precio_costo: request.precio_costo,
            vigente_hasta,
            is_active: request.is_active,
        };
        
        let tarifa = self.tarifa_repository
            .update(id, update)
            .await?;
        
        Ok(self.tarifa_to_response(tarifa))
    }

    /// Desactivar tarifa
    #[instrument(skip(self))]
    pub async fn deactivate_tarifa(&self, id: i32) -> Result<bool, ApplicationError> {
        self.tarifa_repository.deactivate(id).await
    }

    // ========================================================================
    // HELPERS - CONVERSIÓN A RESPONSES
    // ========================================================================

    fn movimiento_to_response(&self, m: MovimientoModel, cuenta_nombre: Option<String>) -> MovimientoResponse {
        MovimientoResponse {
            id: m.id,
            id_cuenta: m.id_cuenta,
            cuenta_nombre,
            tipo: m.tipo,
            monto: m.monto,
            concepto: m.concepto,
            referencia_tipo: m.referencia_tipo,
            referencia_id: m.referencia_id,
            fecha_movimiento: m.fecha_movimiento,
            saldo_anterior: m.saldo_anterior,
            saldo_posterior: m.saldo_posterior,
            notas: m.notas,
            comprobante_url: m.comprobante_url,
            created_at: m.created_at,
        }
    }

    async fn pago_file_to_response(&self, p: PagoFileModel, agencia_nombre: Option<String>) -> PagoFileResponse {
        // Obtener código del file
        let file_code = if let Ok(Some(file)) = self.file_repository.find_by_id(p.id_file).await {
            file.file_code
        } else {
            None
        };
        
        let monto_pendiente = &p.monto_total - &p.monto_pagado;
        
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
            verificador_nombre: None, // TODO: Obtener nombre del verificador
            verificado_at: p.verificado_at,
            notas: p.notas,
            created_at: p.created_at,
        }
    }

    fn pago_proveedor_to_response(&self, p: PagoProveedorModel) -> PagoProveedorResponse {
        // Determinar el ID del proveedor según el tipo
        let proveedor_id = match p.tipo_proveedor.as_str() {
            "transporte" => p.id_transporte.unwrap_or(0),
            "restaurante" => p.id_restaurante.unwrap_or(0),
            "guia" => p.id_guia.unwrap_or(0),
            _ => 0,
        };
        
        PagoProveedorResponse {
            id: p.id,
            tipo_proveedor: p.tipo_proveedor,
            proveedor_id,
            proveedor_nombre: None, // TODO: Obtener nombre del proveedor
            id_file_tour: p.id_file_tour,
            file_code: None, // TODO: Obtener código del file
            tour_nombre: None, // TODO: Obtener nombre del tour
            monto: p.monto,
            estado: p.estado,
            fecha_pago: p.fecha_pago,
            comprobante_url: p.comprobante_url,
            notas: p.notas,
            created_at: p.created_at,
            pagado_por: None, // TODO: Obtener nombre del pagador
        }
    }

    fn tarifa_to_response(&self, t: TarifaServicioModel) -> TarifaServicioResponse {
        TarifaServicioResponse {
            id: t.id,
            tipo_servicio: t.tipo_servicio,
            id_servicio: t.id_servicio,
            servicio_nombre: None, // TODO: Obtener nombre del servicio
            precio_venta: t.precio_venta,
            precio_costo: t.precio_costo,
            margen: t.margen,
            vigente_desde: t.vigente_desde.to_string(),
            vigente_hasta: t.vigente_hasta.map(|d| d.to_string()),
            is_active: t.is_active,
        }
    }
}
