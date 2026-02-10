//! Service para gestión de Saldos a Favor, Cancelaciones y No Shows
//!
//! Lógica de negocio:
//! - Cancelación normal (antes de 8PM del día anterior al file): todo el monto pagado → saldo a favor
//! - No-show (después de 8PM): solo restaurantes + entradas → saldo a favor, el resto → operador

use std::sync::Arc;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use chrono::{Utc, NaiveTime};
use tracing::{info, instrument};

use crate::application::dtos::{
    CancelarFileRequest, CancelacionResponse, RegistrarNoShowRequest,
    SaldoFavorResponse, SaldoFavorDashboard,
    MovimientoSaldoFavorResponse, NoShowResponse,
    UsarSaldoFavorRequest,
};
use crate::application::ports::{FileRepositoryPort, PagoFileRepositoryPort};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::{
    NewCancelacionModel, NewMovimientoSaldoFavorModel, NewNoShowModel,
    UpdatePagoFileModel,
};
use crate::infrastructure::persistence::repositories::SaldoFavorRepositoryPort;

/// Hora límite: 20:40 (8:40 PM) para cancelaciones normales
const HORA_LIMITE_HORA: u32 = 20;
const HORA_LIMITE_MIN: u32 = 40;

pub struct SaldoFavorService {
    saldo_repo: Arc<dyn SaldoFavorRepositoryPort>,
    file_repo: Arc<dyn FileRepositoryPort>,
    pago_file_repo: Arc<dyn PagoFileRepositoryPort>,
}

impl SaldoFavorService {
    pub fn new(
        saldo_repo: Arc<dyn SaldoFavorRepositoryPort>,
        file_repo: Arc<dyn FileRepositoryPort>,
        pago_file_repo: Arc<dyn PagoFileRepositoryPort>,
    ) -> Self {
        Self { saldo_repo, file_repo, pago_file_repo }
    }
    
    /// Cancelación normal de un file (agencia cancela antes de 8PM del día anterior)
    /// Todo el monto pagado se convierte en saldo a favor
    #[instrument(skip(self))]
    pub async fn cancelar_file(&self, request: CancelarFileRequest, user_id: i32) -> Result<CancelacionResponse, ApplicationError> {
        // 1. Obtener el file
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        
        // 2. Verificar que se puede cancelar
        if file.status == "cancelado" || file.status == "anulado" || file.status == "no_show" {
            return Err(ApplicationError::Validation("El file ya esta cancelado, anulado o marcado como no-show".to_string()));
        }
        
        let allowed_statuses = ["pendiente", "reservado", "confirmado", "asignado"];
        if !allowed_statuses.contains(&file.status.as_str()) {
            return Err(ApplicationError::Validation(
                format!("No se puede cancelar un file con status '{}'. Solo se permite cancelar files en estado: pendiente, reservado, confirmado, asignado", file.status)
            ));
        }
        
        // 3. Verificar que no haya cancelación previa
        if self.saldo_repo.find_cancelacion_by_file(request.id_file).await?.is_some() {
            return Err(ApplicationError::Validation("El file ya tiene una cancelación registrada".to_string()));
        }
        
        // 4. Verificar tiempo: debe ser antes de las 8PM del día anterior al inicio del file
        let costs = self.saldo_repo.calculate_file_costs(request.id_file).await?;
        let fecha_inicio = costs.fecha_inicio_min
            .unwrap_or(file.fecha_inicio);
        
        let now = Utc::now();
        let limite = fecha_inicio
            .pred_opt() // día anterior
            .unwrap_or(fecha_inicio)
            .and_time(NaiveTime::from_hms_opt(HORA_LIMITE_HORA, HORA_LIMITE_MIN, 0).unwrap())
            .and_utc();
        
        if now >= limite {
            return Err(ApplicationError::Validation(
                format!("La hora límite para cancelar este file era las {}:{:02} del día anterior ({}). Para registrar un no-show, contacte al administrador.", 
                    HORA_LIMITE_HORA, HORA_LIMITE_MIN, fecha_inicio.pred_opt().unwrap_or(fecha_inicio))
            ));
        }
        
        // 5. En cancelación normal: todo el pagado va a saldo a favor
        let monto_total = BigDecimal::from_f64(file.monto_total.to_f64().unwrap_or(0.0)).unwrap_or_default();
        let monto_pagado = BigDecimal::from_f64(file.monto_pagado.to_f64().unwrap_or(0.0)).unwrap_or_default();
        let monto_saldo_favor = monto_pagado.clone();
        let monto_operador = BigDecimal::from(0);
        
        // 6. Crear la cancelación
        let cancelacion = self.saldo_repo.create_cancelacion(NewCancelacionModel {
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_total_file: monto_total.clone(),
            monto_pagado: monto_pagado.clone(),
            monto_saldo_favor: monto_saldo_favor.clone(),
            monto_operador: monto_operador.clone(),
            tipo_cancelacion: "cancelacion".to_string(),
            motivo: request.motivo,
            notas: request.notas,
            created_by: Some(user_id),
        }).await?;
        
        // 7. Actualizar el status del file a 'cancelado'
        let mut file_to_update = file.clone();
        file_to_update.status = "cancelado".to_string();
        self.file_repo.update(&file_to_update).await?;
        
        // 7b. Actualizar el pago del file a estado 'cancelado'
        if let Ok(Some(pago)) = self.pago_file_repo.find_by_file(request.id_file).await {
            let update = UpdatePagoFileModel {
                estado: Some("cancelado"),
                ..Default::default()
            };
            self.pago_file_repo.update(pago.id, update).await?;
        }
        
        // 8. Si hay saldo a favor, actualizar el saldo de la agencia
        if monto_saldo_favor > BigDecimal::from(0) {
            self.registrar_ingreso_saldo(file.id_agencia, &monto_saldo_favor, cancelacion.id, &file, user_id).await?;
        }
        
        info!("File {} cancelado (normal). Saldo a favor: {}", request.id_file, monto_saldo_favor);
        
        Ok(CancelacionResponse {
            id: cancelacion.id,
            id_file: cancelacion.id_file,
            id_agencia: cancelacion.id_agencia,
            monto_total_file: cancelacion.monto_total_file.to_f64().unwrap_or(0.0),
            monto_pagado: cancelacion.monto_pagado.to_f64().unwrap_or(0.0),
            monto_saldo_favor: cancelacion.monto_saldo_favor.to_f64().unwrap_or(0.0),
            monto_operador: cancelacion.monto_operador.to_f64().unwrap_or(0.0),
            tipo_cancelacion: cancelacion.tipo_cancelacion,
            motivo: cancelacion.motivo,
            notas: cancelacion.notas,
            created_at: cancelacion.created_at,
            created_by: cancelacion.created_by,
            file_code: file.file_code,
            agencia_nombre: None,
        })
    }
    
    /// Registrar un no-show (solo admin, después de 8PM)
    /// Solo restaurantes + entradas van a saldo a favor, el resto al operador
    #[instrument(skip(self))]
    pub async fn registrar_no_show(&self, request: RegistrarNoShowRequest, user_id: i32) -> Result<NoShowResponse, ApplicationError> {
        // 1. Obtener el file
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        
        // 2. Verificar que se puede registrar no-show
        if file.status == "cancelado" || file.status == "anulado" || file.status == "no_show" {
            return Err(ApplicationError::Validation("El file ya esta cancelado, anulado o marcado como no-show".to_string()));
        }
        
        // 3. Verificar que no haya cancelación previa
        if self.saldo_repo.find_cancelacion_by_file(request.id_file).await?.is_some() {
            return Err(ApplicationError::Validation("El file ya tiene una cancelación/no-show registrado".to_string()));
        }
        
        // 4. Calcular costos del file
        let costs = self.saldo_repo.calculate_file_costs(request.id_file).await?;
        let fecha_inicio = costs.fecha_inicio_min
            .unwrap_or(file.fecha_inicio);
        
        let monto_total = BigDecimal::from_f64(file.monto_total.to_f64().unwrap_or(0.0)).unwrap_or_default();
        let monto_pagado = BigDecimal::from_f64(file.monto_pagado.to_f64().unwrap_or(0.0)).unwrap_or_default();
        
        // 5. Calcular saldo a favor (solo restaurantes + entradas)
        let monto_restaurantes = costs.monto_restaurantes;
        let monto_entradas = costs.monto_entradas;
        let monto_saldo_favor = &monto_restaurantes + &monto_entradas;
        
        // El monto_saldo_favor no puede exceder el monto_pagado
        let monto_saldo_favor = if monto_saldo_favor > monto_pagado {
            monto_pagado.clone()
        } else {
            monto_saldo_favor
        };
        
        // Lo que queda para el operador
        let monto_operador = &monto_pagado - &monto_saldo_favor;
        
        // 6. Crear la cancelación (tipo no_show)
        let cancelacion = self.saldo_repo.create_cancelacion(NewCancelacionModel {
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_total_file: monto_total.clone(),
            monto_pagado: monto_pagado.clone(),
            monto_saldo_favor: monto_saldo_favor.clone(),
            monto_operador: monto_operador.clone(),
            tipo_cancelacion: "no_show".to_string(),
            motivo: request.motivo,
            notas: request.notas.clone(),
            created_by: Some(user_id),
        }).await?;
        
        // 7. Crear el registro de no_show con desglose
        let no_show = self.saldo_repo.create_no_show(NewNoShowModel {
            id_cancelacion: cancelacion.id,
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_restaurantes: monto_restaurantes.clone(),
            monto_entradas: monto_entradas.clone(),
            monto_saldo_favor: monto_saldo_favor.clone(),
            monto_operador: monto_operador.clone(),
            fecha_inicio_file: fecha_inicio,
            hora_corte: Utc::now(),
            notas: request.notas,
            created_by: Some(user_id),
        }).await?;
        
        // 8. Actualizar el status del file a 'no_show'
        let mut file_to_update = file.clone();
        file_to_update.status = "no_show".to_string();
        self.file_repo.update(&file_to_update).await?;
        
        // 8b. Actualizar el pago del file a estado 'no_show'
        if let Ok(Some(pago)) = self.pago_file_repo.find_by_file(request.id_file).await {
            let update = UpdatePagoFileModel {
                estado: Some("no_show"),
                ..Default::default()
            };
            self.pago_file_repo.update(pago.id, update).await?;
        }
        
        // 9. Si hay saldo a favor, actualizar
        if monto_saldo_favor > BigDecimal::from(0) {
            self.registrar_ingreso_saldo(file.id_agencia, &monto_saldo_favor, cancelacion.id, &file, user_id).await?;
        }
        
        info!("File {} registrado como no-show. Saldo favor: {}, Operador: {}", 
            request.id_file, 
            monto_saldo_favor.to_f64().unwrap_or(0.0), 
            monto_operador.to_f64().unwrap_or(0.0));
        
        Ok(NoShowResponse {
            id: no_show.id,
            id_cancelacion: no_show.id_cancelacion,
            id_file: no_show.id_file,
            id_agencia: no_show.id_agencia,
            monto_restaurantes: no_show.monto_restaurantes.to_f64().unwrap_or(0.0),
            monto_entradas: no_show.monto_entradas.to_f64().unwrap_or(0.0),
            monto_saldo_favor: no_show.monto_saldo_favor.to_f64().unwrap_or(0.0),
            monto_operador: no_show.monto_operador.to_f64().unwrap_or(0.0),
            fecha_inicio_file: no_show.fecha_inicio_file,
            hora_corte: no_show.hora_corte,
            notas: no_show.notas,
            created_at: no_show.created_at,
            created_by: no_show.created_by,
            file_code: file.file_code,
            agencia_nombre: None,
        })
    }
    
    /// Helper: registrar ingreso de saldo a favor
    async fn registrar_ingreso_saldo(
        &self,
        id_agencia: i32,
        monto_saldo_favor: &BigDecimal,
        id_cancelacion: i32,
        file: &crate::domain::entities::File,
        user_id: i32,
    ) -> Result<(), ApplicationError> {
        let saldo = self.saldo_repo.get_saldo_by_agencia(id_agencia).await?
            .ok_or_else(|| ApplicationError::NotFound(
                format!("Saldo a favor no encontrado para agencia {}", id_agencia)
            ))?;
        
        let saldo_anterior = saldo.saldo_disponible.clone();
        let nuevo_disponible = &saldo.saldo_disponible + monto_saldo_favor;
        let nuevo_total = &saldo.saldo_total_generado + monto_saldo_favor;
        
        self.saldo_repo.update_saldo(
            saldo.id,
            nuevo_disponible.clone(),
            saldo.saldo_utilizado.clone(),
            nuevo_total,
        ).await?;
        
        self.saldo_repo.create_movimiento(NewMovimientoSaldoFavorModel {
            id_saldo_favor: saldo.id,
            id_agencia,
            tipo: "ingreso".to_string(),
            monto: monto_saldo_favor.clone(),
            id_cancelacion: Some(id_cancelacion),
            id_file_destino: None,
            id_pago_file: None,
            saldo_anterior,
            saldo_posterior: nuevo_disponible,
            concepto: Some(format!("Saldo a favor por cancelación de file #{}", file.file_code.as_deref().unwrap_or("?"))),
            created_by: Some(user_id),
        }).await?;
        
        Ok(())
    }
    
    /// Usa saldo a favor para pagar un file
    #[instrument(skip(self))]
    pub async fn usar_saldo(&self, request: UsarSaldoFavorRequest, user_id: i32) -> Result<MovimientoSaldoFavorResponse, ApplicationError> {
        let monto = BigDecimal::from_f64(request.monto)
            .ok_or_else(|| ApplicationError::Validation("Monto inválido".to_string()))?;
        
        if monto <= BigDecimal::from(0) {
            return Err(ApplicationError::Validation("El monto debe ser mayor a 0".to_string()));
        }
        
        // Verificar que el file destino existe
        let file_destino = self.file_repo.find_by_id(request.id_file_destino).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file_destino)))?;
        
        // Verificar que el file pertenece a la misma agencia
        if file_destino.id_agencia != request.id_agencia {
            return Err(ApplicationError::Validation("El file destino no pertenece a esta agencia".to_string()));
        }
        
        // Obtener el saldo
        let saldo = self.saldo_repo.get_saldo_by_agencia(request.id_agencia).await?
            .ok_or_else(|| ApplicationError::NotFound("Saldo a favor no encontrado".to_string()))?;
        
        // Verificar saldo suficiente
        if saldo.saldo_disponible < monto {
            return Err(ApplicationError::Validation(
                format!("Saldo insuficiente. Disponible: {}, Solicitado: {}", 
                    saldo.saldo_disponible.to_f64().unwrap_or(0.0), request.monto)
            ));
        }
        
        let saldo_anterior = saldo.saldo_disponible.clone();
        let nuevo_disponible = &saldo.saldo_disponible - &monto;
        let nuevo_utilizado = &saldo.saldo_utilizado + &monto;
        
        // Actualizar saldo
        self.saldo_repo.update_saldo(
            saldo.id,
            nuevo_disponible.clone(),
            nuevo_utilizado,
            saldo.saldo_total_generado.clone(),
        ).await?;
        
        // Registrar movimiento de uso
        let movimiento = self.saldo_repo.create_movimiento(NewMovimientoSaldoFavorModel {
            id_saldo_favor: saldo.id,
            id_agencia: request.id_agencia,
            tipo: "uso".to_string(),
            monto: monto.clone(),
            id_cancelacion: None,
            id_file_destino: Some(request.id_file_destino),
            id_pago_file: request.id_pago_file,
            saldo_anterior: saldo_anterior.clone(),
            saldo_posterior: nuevo_disponible.clone(),
            concepto: request.concepto.or(Some(format!(
                "Uso de saldo a favor en file #{}", 
                file_destino.file_code.as_deref().unwrap_or("?")
            ))),
            created_by: Some(user_id),
        }).await?;
        
        info!("Saldo a favor usado: {} para file {} (agencia {})", request.monto, request.id_file_destino, request.id_agencia);
        
        Ok(MovimientoSaldoFavorResponse {
            id: movimiento.id,
            id_agencia: movimiento.id_agencia,
            tipo: movimiento.tipo,
            monto: movimiento.monto.to_f64().unwrap_or(0.0),
            id_cancelacion: movimiento.id_cancelacion,
            id_file_destino: movimiento.id_file_destino,
            id_pago_file: movimiento.id_pago_file,
            saldo_anterior: saldo_anterior.to_f64().unwrap_or(0.0),
            saldo_posterior: nuevo_disponible.to_f64().unwrap_or(0.0),
            concepto: movimiento.concepto,
            created_at: movimiento.created_at,
            created_by: movimiento.created_by,
            file_code_origen: None,
            file_code_destino: file_destino.file_code,
        })
    }
    
    /// Obtiene el dashboard de saldo a favor para una agencia
    #[instrument(skip(self))]
    pub async fn get_dashboard(&self, id_agencia: i32) -> Result<SaldoFavorDashboard, ApplicationError> {
        let saldo = self.saldo_repo.get_saldo_by_agencia(id_agencia).await?
            .ok_or_else(|| ApplicationError::NotFound("Saldo a favor no encontrado".to_string()))?;
        
        let cancelaciones = self.saldo_repo.list_cancelaciones(Some(id_agencia), 10, 0).await?;
        let movimientos = self.saldo_repo.list_movimientos(Some(id_agencia), None, 10, 0).await?;
        let total_cancelaciones = self.saldo_repo.count_cancelaciones(Some(id_agencia)).await?;
        let total_no_shows = self.saldo_repo.count_no_shows(Some(id_agencia)).await?;
        
        Ok(SaldoFavorDashboard {
            saldo: SaldoFavorResponse {
                id: saldo.id,
                id_agencia: saldo.id_agencia,
                agencia_nombre: None,
                saldo_disponible: saldo.saldo_disponible.to_f64().unwrap_or(0.0),
                saldo_utilizado: saldo.saldo_utilizado.to_f64().unwrap_or(0.0),
                saldo_total_generado: saldo.saldo_total_generado.to_f64().unwrap_or(0.0),
                updated_at: saldo.updated_at,
            },
            cancelaciones_recientes: cancelaciones,
            movimientos_recientes: movimientos,
            total_cancelaciones,
            total_no_shows,
        })
    }
    
    /// Lista todos los saldos (para admin)
    pub async fn list_all_saldos(&self) -> Result<Vec<SaldoFavorResponse>, ApplicationError> {
        self.saldo_repo.list_all_saldos().await
    }
    
    /// Lista cancelaciones con filtros
    pub async fn list_cancelaciones(&self, id_agencia: Option<i32>, page: i64, per_page: i64) -> Result<Vec<CancelacionResponse>, ApplicationError> {
        let offset = (page - 1) * per_page;
        self.saldo_repo.list_cancelaciones(id_agencia, per_page, offset).await
    }
    
    /// Lista movimientos con filtros
    pub async fn list_movimientos(&self, id_agencia: Option<i32>, tipo: Option<&str>, page: i64, per_page: i64) -> Result<Vec<MovimientoSaldoFavorResponse>, ApplicationError> {
        let offset = (page - 1) * per_page;
        self.saldo_repo.list_movimientos(id_agencia, tipo, per_page, offset).await
    }
    
    /// Lista no-shows con filtros
    pub async fn list_no_shows(&self, id_agencia: Option<i32>, page: i64, per_page: i64) -> Result<Vec<NoShowResponse>, ApplicationError> {
        let offset = (page - 1) * per_page;
        self.saldo_repo.list_no_shows(id_agencia, per_page, offset).await
    }
    
    /// Saldo de una agencia
    pub async fn get_saldo_agencia(&self, id_agencia: i32) -> Result<SaldoFavorResponse, ApplicationError> {
        let saldo = self.saldo_repo.get_saldo_by_agencia(id_agencia).await?
            .ok_or_else(|| ApplicationError::NotFound("Saldo no encontrado".to_string()))?;
        
        Ok(SaldoFavorResponse {
            id: saldo.id,
            id_agencia: saldo.id_agencia,
            agencia_nombre: None,
            saldo_disponible: saldo.saldo_disponible.to_f64().unwrap_or(0.0),
            saldo_utilizado: saldo.saldo_utilizado.to_f64().unwrap_or(0.0),
            saldo_total_generado: saldo.saldo_total_generado.to_f64().unwrap_or(0.0),
            updated_at: saldo.updated_at,
        })
    }
}
