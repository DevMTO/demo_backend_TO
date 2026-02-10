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
};
use crate::domain::errors::ApplicationError;
use crate::domain::entities::{
    UserRole, NotificationType, NotificationCategory, NotificationPriority,
};
use crate::infrastructure::persistence::models::{
    NewPagoProveedorModel,
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
}

impl ContabilidadService {
    pub fn new(
        pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
        pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
        agencia_repository: Arc<dyn AgenciaRepositoryPort>,
        file_repository: Arc<dyn FileRepositoryPort>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            pago_file_repository,
            pago_proveedor_repository,
            agencia_repository,
            file_repository,
            notification_service,
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
        
        // Separar files pendientes y ultimos pagos
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
        
        // Obtener nombre de agencia para notificacion
        let agencia_nombre = if let Ok(Some(agencia)) = self.agencia_repository.find_by_id(pago_actualizado.id_agencia).await {
            Some(agencia.nombre.clone())
        } else {
            None
        };
        
        // Notificar a los admins del pago registrado
        let estado_texto = if nuevo_estado == "pagado" { "completo" } else { "parcial" };
        let titulo_notif = if nuevo_estado == "pagado" {
            "Pago Completo Registrado"
        } else {
            "Pago Parcial Registrado"
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
            warn!("Error al enviar notificacion de pago registrado: {}", e);
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
        let _pago = self.pago_file_repository
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
        
        let comprobante_url: Option<&str> = request.comprobante_url.as_deref();
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
        
        info!("Pago a proveedor {} marcado como pagado por {}", id_pago_proveedor, pagado_by);
        
        Ok(self.pago_proveedor_to_response(pago_actualizado))
    }

    // ========================================================================
    // HELPERS - CONVERSION A RESPONSES
    // ========================================================================

    async fn pago_file_to_response(&self, p: PagoFileModel, agencia_nombre: Option<String>) -> PagoFileResponse {
        // Obtener codigo del file
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
            verificador_nombre: None,
            verificado_at: p.verificado_at,
            notas: p.notas,
            created_at: p.created_at,
        }
    }

    fn pago_proveedor_to_response(&self, p: PagoProveedorModel) -> PagoProveedorResponse {
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
            proveedor_nombre: None,
            id_file_tour: p.id_file_tour,
            file_code: None,
            tour_nombre: None,
            monto: p.monto,
            estado: p.estado,
            fecha_pago: p.fecha_pago,
            comprobante_url: p.comprobante_url,
            notas: p.notas,
            created_at: p.created_at,
            pagado_por: None,
        }
    }
}