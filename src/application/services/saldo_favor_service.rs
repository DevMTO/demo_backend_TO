//! Service para gestión de Saldos a Favor y Cancelaciones

use std::sync::Arc;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use tracing::{info, instrument};

use crate::application::dtos::{
    CancelarFileRequest, CancelacionResponse,
    SaldoFavorResponse, SaldoFavorDashboard,
    MovimientoSaldoFavorResponse,
    UsarSaldoFavorRequest,
};
use crate::application::ports::FileRepositoryPort;
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::models::{
    NewCancelacionModel, NewMovimientoSaldoFavorModel,
};
use crate::infrastructure::persistence::repositories::SaldoFavorRepositoryPort;

pub struct SaldoFavorService {
    saldo_repo: Arc<dyn SaldoFavorRepositoryPort>,
    file_repo: Arc<dyn FileRepositoryPort>,
}

impl SaldoFavorService {
    pub fn new(
        saldo_repo: Arc<dyn SaldoFavorRepositoryPort>,
        file_repo: Arc<dyn FileRepositoryPort>,
    ) -> Self {
        Self { saldo_repo, file_repo }
    }
    
    /// Cancela un file y genera saldo a favor para la agencia
    #[instrument(skip(self))]
    pub async fn cancelar_file(&self, request: CancelarFileRequest, user_id: i32) -> Result<CancelacionResponse, ApplicationError> {
        // 1. Obtener el file
        let file = self.file_repo.find_by_id(request.id_file).await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", request.id_file)))?;
        
        // 2. Verificar que se puede cancelar
        if file.status == "cancelado" || file.status == "anulado" {
            return Err(ApplicationError::Validation("El file ya está cancelado o anulado".to_string()));
        }
        
        let allowed_statuses = ["pendiente", "reservado", "confirmado", "asignado"];
        if !allowed_statuses.contains(&file.status.as_str()) {
            return Err(ApplicationError::Validation(
                format!("No se puede cancelar un file con status '{}'. Solo se permite cancelar files en estado: pendiente, reservado, confirmado, asignado", file.status)
            ));
        }
        
        // 3. Verificar que no haya cancelación previa
        if let Some(_) = self.saldo_repo.find_cancelacion_by_file(request.id_file).await? {
            return Err(ApplicationError::Validation("El file ya tiene una cancelación registrada".to_string()));
        }
        
        // 4. Calcular montos
        let monto_total = BigDecimal::from_f64(file.monto_total.to_f64().unwrap_or(0.0)).unwrap_or_default();
        let monto_pagado = BigDecimal::from_f64(file.monto_pagado.to_f64().unwrap_or(0.0)).unwrap_or_default();
        
        let porcentaje_penalidad = request.porcentaje_penalidad.unwrap_or(0.0);
        let penalidad_factor = BigDecimal::from_f64(porcentaje_penalidad / 100.0).unwrap_or_default();
        let monto_penalidad = &monto_pagado * &penalidad_factor;
        let monto_saldo_favor = &monto_pagado - &monto_penalidad;
        
        // Determinar tipo de cancelación
        let tipo_cancelacion = if porcentaje_penalidad > 0.0 {
            "fuera_tiempo".to_string()
        } else {
            "en_tiempo".to_string()
        };
        
        // 5. Crear la cancelación
        let cancelacion = self.saldo_repo.create_cancelacion(NewCancelacionModel {
            id_file: request.id_file,
            id_agencia: file.id_agencia,
            monto_total_file: monto_total.clone(),
            monto_pagado: monto_pagado.clone(),
            monto_penalidad: monto_penalidad.clone(),
            monto_saldo_favor: monto_saldo_favor.clone(),
            tipo_cancelacion,
            hora_limite_cancelacion: None,
            motivo: request.motivo,
            notas: request.notas,
            created_by: Some(user_id),
        }).await?;
        
        // 6. Actualizar el status del file a 'cancelado'
        let mut file_to_update = file.clone();
        file_to_update.status = "cancelado".to_string();
        self.file_repo.update(&file_to_update).await?;
        
        // 7. Si hay saldo a favor, actualizar el saldo de la agencia
        if monto_saldo_favor > BigDecimal::from(0) {
            let saldo = self.saldo_repo.get_saldo_by_agencia(file.id_agencia).await?
                .ok_or_else(|| ApplicationError::NotFound(
                    format!("Saldo a favor no encontrado para agencia {}", file.id_agencia)
                ))?;
            
            let saldo_anterior = saldo.saldo_disponible.clone();
            let nuevo_disponible = &saldo.saldo_disponible + &monto_saldo_favor;
            let nuevo_total = &saldo.saldo_total_generado + &monto_saldo_favor;
            
            self.saldo_repo.update_saldo(
                saldo.id,
                nuevo_disponible.clone(),
                saldo.saldo_utilizado.clone(),
                nuevo_total,
            ).await?;
            
            // 8. Registrar movimiento de ingreso
            self.saldo_repo.create_movimiento(NewMovimientoSaldoFavorModel {
                id_saldo_favor: saldo.id,
                id_agencia: file.id_agencia,
                tipo: "ingreso".to_string(),
                monto: monto_saldo_favor.clone(),
                id_cancelacion: Some(cancelacion.id),
                id_file_destino: None,
                id_pago_file: None,
                saldo_anterior,
                saldo_posterior: nuevo_disponible,
                concepto: Some(format!("Saldo a favor por cancelación de file #{}", file.file_code.as_deref().unwrap_or("?"))),
                created_by: Some(user_id),
            }).await?;
        }
        
        info!("File {} cancelado. Saldo a favor generado: {}", request.id_file, monto_saldo_favor);
        
        Ok(CancelacionResponse {
            id: cancelacion.id,
            id_file: cancelacion.id_file,
            id_agencia: cancelacion.id_agencia,
            monto_total_file: cancelacion.monto_total_file.to_f64().unwrap_or(0.0),
            monto_pagado: cancelacion.monto_pagado.to_f64().unwrap_or(0.0),
            monto_penalidad: cancelacion.monto_penalidad.to_f64().unwrap_or(0.0),
            monto_saldo_favor: cancelacion.monto_saldo_favor.to_f64().unwrap_or(0.0),
            tipo_cancelacion: cancelacion.tipo_cancelacion,
            hora_limite_cancelacion: cancelacion.hora_limite_cancelacion,
            motivo: cancelacion.motivo,
            notas: cancelacion.notas,
            created_at: cancelacion.created_at,
            created_by: cancelacion.created_by,
            file_code: file.file_code,
            agencia_nombre: None,
        })
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
