use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::dsl::sum;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{PagoRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Pago, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{PagoModel, NewPagoModel, UpdatePagoModel},
    schema::pagos,
};

pub struct PostgresPagoRepository {
    pool: DatabasePool,
}

impl PostgresPagoRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PagoRepositoryPort for PostgresPagoRepository {
    #[instrument(skip(self, pago))]
    async fn create(&self, pago: &Pago) -> Result<Pago, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_pago: NewPagoModel = pago.into();
        
        let result = diesel::insert_into(pagos::table)
            .values(&new_pago)
            .returning(PagoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("✅ Pago creado: {} S/.{} (id: {})", result.tipo_movimiento, result.monto, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Pago>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos::table
            .filter(pagos::id.eq(id))
            .select(PagoModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, pago: &Pago) -> Result<Pago, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let changes = UpdatePagoModel {
            tipo_movimiento: Some(&pago.tipo_movimiento),
            concepto: Some(&pago.concepto),
            monto: Some(pago.monto.clone()),
            metodo_pago: Some(pago.metodo_pago.as_deref()),
            referencia: Some(pago.referencia.as_deref()),
            evidencia: Some(pago.evidencia.clone()),
            fecha_pago: Some(pago.fecha_pago),
            notas: Some(pago.notas.as_deref()),
            updated_by: pago.updated_by,
        };
        
        let result = diesel::update(pagos::table.filter(pagos::id.eq(pago.id)))
            .set(&changes)
            .returning(PagoModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(pagos::table.filter(pagos::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Pago>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = pagos::table
            .order(pagos::fecha_pago.desc())
            .limit(limit)
            .offset(offset)
            .select(PagoModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        pagos::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Pago>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn find_by_file(&self, id_file: i32) -> Result<Vec<Pago>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = pagos::table
            .filter(pagos::id_file.eq(id_file))
            .order(pagos::fecha_pago.desc())
            .select(PagoModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn sum_ingresos_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result: Option<BigDecimal> = pagos::table
            .filter(pagos::id_file.eq(id_file))
            .filter(pagos::tipo_movimiento.eq("ingreso"))
            .select(sum(pagos::monto))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from(0)))
    }
    
    async fn sum_egresos_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result: Option<BigDecimal> = pagos::table
            .filter(pagos::id_file.eq(id_file))
            .filter(pagos::tipo_movimiento.eq("egreso"))
            .select(sum(pagos::monto))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from(0)))
    }
    
    async fn get_balance_by_file(&self, id_file: i32) -> Result<BigDecimal, ApplicationError> {
        let ingresos = self.sum_ingresos_by_file(id_file).await?;
        let egresos = self.sum_egresos_by_file(id_file).await?;
        Ok(ingresos - egresos)
    }
}
