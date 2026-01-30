//! Implementación de repositorios de contabilidad
//!
//! Incluye:
//! - PostgresCuentaRepository
//! - PostgresMovimientoRepository
//! - PostgresPagoFileRepository
//! - PostgresPagoProveedorRepository
//! - PostgresTarifaServicioRepository

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::str::FromStr;
use tracing::{info, instrument};

use crate::application::ports::contabilidad_repository::{
    CuentaRepositoryPort, MovimientoRepositoryPort, PagoFileRepositoryPort,
    PagoProveedorRepositoryPort, TarifaServicioRepositoryPort,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;
use crate::infrastructure::persistence::models::{
    CuentaModel, MovimientoModel, PagoFileModel, PagoProveedorModel, TarifaServicioModel,
    NewCuentaModel, NewMovimientoModel, NewPagoFileModel, NewPagoProveedorModel, NewTarifaServicioModel,
    UpdateCuentaModel, UpdatePagoFileModel, UpdatePagoProveedorModel, UpdateTarifaServicioModel,
};
use crate::infrastructure::persistence::schema::{
    cuentas, movimientos, pagos_files, pagos_proveedores, tarifas_servicios,
};

// ============================================================================
// CUENTA REPOSITORY
// ============================================================================

pub struct PostgresCuentaRepository {
    pool: DatabasePool,
}

impl PostgresCuentaRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CuentaRepositoryPort for PostgresCuentaRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<CuentaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = cuentas::table
            .find(id)
            .first::<CuentaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_admin_account(&self) -> Result<Option<CuentaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = cuentas::table
            .filter(cuentas::tipo.eq("admin"))
            .filter(cuentas::is_active.eq(true))
            .first::<CuentaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_agencia(&self, id_agencia: i32) -> Result<Option<CuentaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = cuentas::table
            .filter(cuentas::tipo.eq("agencia"))
            .filter(cuentas::id_agencia.eq(id_agencia))
            .filter(cuentas::is_active.eq(true))
            .first::<CuentaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_all(&self) -> Result<Vec<CuentaModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = cuentas::table
            .filter(cuentas::is_active.eq(true))
            .order(cuentas::created_at.desc())
            .load::<CuentaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewCuentaModel<'_>) -> Result<CuentaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::insert_into(cuentas::table)
            .values(&data)
            .get_result::<CuentaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Cuenta creada: {} ({})", result.nombre, result.tipo);
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn update_saldo(&self, id: i32, nuevo_saldo: BigDecimal) -> Result<CuentaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(cuentas::table.find(id))
            .set((
                cuentas::saldo_actual.eq(nuevo_saldo),
                cuentas::updated_at.eq(Utc::now()),
            ))
            .get_result::<CuentaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdateCuentaModel<'_>) -> Result<CuentaModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(cuentas::table.find(id))
            .set(&data)
            .get_result::<CuentaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }
}

// ============================================================================
// MOVIMIENTO REPOSITORY
// ============================================================================

pub struct PostgresMovimientoRepository {
    pool: DatabasePool,
}

impl PostgresMovimientoRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MovimientoRepositoryPort for PostgresMovimientoRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<MovimientoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = movimientos::table
            .find(id)
            .first::<MovimientoModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_cuenta(
        &self, 
        id_cuenta: i32, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<MovimientoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = movimientos::table
            .filter(movimientos::id_cuenta.eq(id_cuenta))
            .order(movimientos::fecha_movimiento.desc())
            .limit(limit)
            .offset(offset)
            .load::<MovimientoModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_filtered(
        &self,
        id_cuenta: Option<i32>,
        tipo: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MovimientoModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = movimientos::table
            .into_boxed();
        
        if let Some(cuenta_id) = id_cuenta {
            query = query.filter(movimientos::id_cuenta.eq(cuenta_id));
        }
        
        if let Some(t) = tipo {
            query = query.filter(movimientos::tipo.eq(t));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(movimientos::fecha_movimiento.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(movimientos::fecha_movimiento.le(hasta));
        }
        
        let result = query
            .order(movimientos::fecha_movimiento.desc())
            .limit(limit)
            .offset(offset)
            .load::<MovimientoModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn count_filtered(
        &self,
        id_cuenta: Option<i32>,
        tipo: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
    ) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = movimientos::table
            .into_boxed();
        
        if let Some(cuenta_id) = id_cuenta {
            query = query.filter(movimientos::id_cuenta.eq(cuenta_id));
        }
        
        if let Some(t) = tipo {
            query = query.filter(movimientos::tipo.eq(t));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(movimientos::fecha_movimiento.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(movimientos::fecha_movimiento.le(hasta));
        }
        
        let result = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewMovimientoModel<'_>) -> Result<MovimientoModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::insert_into(movimientos::table)
            .values(&data)
            .get_result::<MovimientoModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Movimiento creado: {} - {} ({})", result.id, result.concepto, result.tipo);
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn sum_ingresos(
        &self, 
        id_cuenta: i32, 
        fecha_desde: DateTime<Utc>, 
        fecha_hasta: DateTime<Utc>
    ) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        use diesel::dsl::sum;
        
        let result: Option<BigDecimal> = movimientos::table
            .filter(movimientos::id_cuenta.eq(id_cuenta))
            .filter(movimientos::tipo.eq("ingreso"))
            .filter(movimientos::fecha_movimiento.ge(fecha_desde))
            .filter(movimientos::fecha_movimiento.le(fecha_hasta))
            .select(sum(movimientos::monto))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from_str("0").unwrap()))
    }

    #[instrument(skip(self))]
    async fn sum_egresos(
        &self, 
        id_cuenta: i32, 
        fecha_desde: DateTime<Utc>, 
        fecha_hasta: DateTime<Utc>
    ) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        use diesel::dsl::sum;
        
        let result: Option<BigDecimal> = movimientos::table
            .filter(movimientos::id_cuenta.eq(id_cuenta))
            .filter(movimientos::tipo.eq("egreso"))
            .filter(movimientos::fecha_movimiento.ge(fecha_desde))
            .filter(movimientos::fecha_movimiento.le(fecha_hasta))
            .select(sum(movimientos::monto))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from_str("0").unwrap()))
    }
}

// ============================================================================
// PAGO FILE REPOSITORY
// ============================================================================

pub struct PostgresPagoFileRepository {
    pool: DatabasePool,
}

impl PostgresPagoFileRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PagoFileRepositoryPort for PostgresPagoFileRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_files::table
            .find(id)
            .first::<PagoFileModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_file(&self, id_file: i32) -> Result<Option<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_files::table
            .filter(pagos_files::id_file.eq(id_file))
            .first::<PagoFileModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_agencia(
        &self, 
        id_agencia: i32, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_files::table
            .filter(pagos_files::id_agencia.eq(id_agencia))
            .order(pagos_files::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<PagoFileModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_filtered(
        &self,
        id_agencia: Option<i32>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_files::table.into_boxed();
        
        if let Some(agencia_id) = id_agencia {
            query = query.filter(pagos_files::id_agencia.eq(agencia_id));
        }
        
        if let Some(e) = estado {
            query = query.filter(pagos_files::estado.eq(e));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(pagos_files::fecha_vencimiento.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(pagos_files::fecha_vencimiento.le(hasta));
        }
        
        let result = query
            .order(pagos_files::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<PagoFileModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn count_filtered(
        &self,
        id_agencia: Option<i32>,
        estado: Option<&str>,
        fecha_desde: Option<NaiveDate>,
        fecha_hasta: Option<NaiveDate>,
    ) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_files::table.into_boxed();
        
        if let Some(agencia_id) = id_agencia {
            query = query.filter(pagos_files::id_agencia.eq(agencia_id));
        }
        
        if let Some(e) = estado {
            query = query.filter(pagos_files::estado.eq(e));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(pagos_files::fecha_vencimiento.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(pagos_files::fecha_vencimiento.le(hasta));
        }
        
        let result = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewPagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::insert_into(pagos_files::table)
            .values(&data)
            .get_result::<PagoFileModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Pago file creado: {} - file {}", result.id, result.id_file);
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdatePagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(pagos_files::table.find(id))
            .set(&data)
            .get_result::<PagoFileModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn sum_pendiente_cobrar(&self) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        use diesel::dsl::sum;
        
        // Sumar (monto_total - monto_pagado) de todos los files pendientes/parciales
        let result: Option<BigDecimal> = pagos_files::table
            .filter(pagos_files::estado.eq_any(vec!["pendiente", "parcial", "vencido"]))
            .select(sum(pagos_files::monto_total - pagos_files::monto_pagado))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from_str("0").unwrap()))
    }

    #[instrument(skip(self))]
    async fn sum_pendiente_agencia(&self, id_agencia: i32) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        use diesel::dsl::sum;
        
        let result: Option<BigDecimal> = pagos_files::table
            .filter(pagos_files::id_agencia.eq(id_agencia))
            .filter(pagos_files::estado.eq_any(vec!["pendiente", "parcial", "vencido"]))
            .select(sum(pagos_files::monto_total - pagos_files::monto_pagado))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from_str("0").unwrap()))
    }

    #[instrument(skip(self))]
    async fn count_pendientes(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_files::table
            .filter(pagos_files::estado.eq_any(vec!["pendiente", "parcial", "vencido"]))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }
}

// ============================================================================
// PAGO PROVEEDOR REPOSITORY
// ============================================================================

pub struct PostgresPagoProveedorRepository {
    pool: DatabasePool,
}

impl PostgresPagoProveedorRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PagoProveedorRepositoryPort for PostgresPagoProveedorRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_proveedores::table
            .find(id)
            .first::<PagoProveedorModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_proveedor(
        &self,
        tipo_proveedor: &str,
        id_proveedor: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_proveedores::table
            .filter(pagos_proveedores::tipo_proveedor.eq(tipo_proveedor))
            .into_boxed();
        
        // Filtrar según el tipo de proveedor
        query = match tipo_proveedor {
            "transporte" => query.filter(pagos_proveedores::id_transporte.eq(id_proveedor)),
            "restaurante" => query.filter(pagos_proveedores::id_restaurante.eq(id_proveedor)),
            "guia" => query.filter(pagos_proveedores::id_guia.eq(id_proveedor)),
            _ => return Err(ApplicationError::Validation(format!("Tipo de proveedor inválido: {}", tipo_proveedor))),
        };
        
        let result = query
            .order(pagos_proveedores::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<PagoProveedorModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_filtered(
        &self,
        tipo_proveedor: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_proveedores::table.into_boxed();
        
        if let Some(tp) = tipo_proveedor {
            query = query.filter(pagos_proveedores::tipo_proveedor.eq(tp));
        }
        
        if let Some(e) = estado {
            query = query.filter(pagos_proveedores::estado.eq(e));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(pagos_proveedores::created_at.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(pagos_proveedores::created_at.le(hasta));
        }
        
        let result = query
            .order(pagos_proveedores::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<PagoProveedorModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn count_filtered(
        &self,
        tipo_proveedor: Option<&str>,
        estado: Option<&str>,
        fecha_desde: Option<DateTime<Utc>>,
        fecha_hasta: Option<DateTime<Utc>>,
    ) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_proveedores::table.into_boxed();
        
        if let Some(tp) = tipo_proveedor {
            query = query.filter(pagos_proveedores::tipo_proveedor.eq(tp));
        }
        
        if let Some(e) = estado {
            query = query.filter(pagos_proveedores::estado.eq(e));
        }
        
        if let Some(desde) = fecha_desde {
            query = query.filter(pagos_proveedores::created_at.ge(desde));
        }
        
        if let Some(hasta) = fecha_hasta {
            query = query.filter(pagos_proveedores::created_at.le(hasta));
        }
        
        let result = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewPagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::insert_into(pagos_proveedores::table)
            .values(&data)
            .get_result::<PagoProveedorModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Pago proveedor creado: {} - {} ({})", result.id, result.tipo_proveedor, result.estado);
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdatePagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(pagos_proveedores::table.find(id))
            .set(&data)
            .get_result::<PagoProveedorModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn sum_pendiente_pagar(&self) -> Result<BigDecimal, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        use diesel::dsl::sum;
        
        let result: Option<BigDecimal> = pagos_proveedores::table
            .filter(pagos_proveedores::estado.eq("pendiente"))
            .select(sum(pagos_proveedores::monto))
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| BigDecimal::from_str("0").unwrap()))
    }

    #[instrument(skip(self))]
    async fn count_pendientes(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = pagos_proveedores::table
            .filter(pagos_proveedores::estado.eq("pendiente"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_file_relation(
        &self,
        tipo_proveedor: &str,
        id_file_vehiculo: Option<i32>,
        id_file_restaurante: Option<i32>,
        id_file_guia: Option<i32>,
    ) -> Result<Option<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = pagos_proveedores::table
            .filter(pagos_proveedores::tipo_proveedor.eq(tipo_proveedor))
            .into_boxed();
        
        if let Some(id) = id_file_vehiculo {
            query = query.filter(pagos_proveedores::id_file_vehiculo.eq(id));
        }
        
        if let Some(id) = id_file_restaurante {
            query = query.filter(pagos_proveedores::id_file_restaurante.eq(id));
        }
        
        if let Some(id) = id_file_guia {
            query = query.filter(pagos_proveedores::id_file_guia.eq(id));
        }
        
        let result = query
            .first::<PagoProveedorModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }
}

// ============================================================================
// TARIFA SERVICIO REPOSITORY
// ============================================================================

pub struct PostgresTarifaServicioRepository {
    pool: DatabasePool,
}

impl PostgresTarifaServicioRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TarifaServicioRepositoryPort for PostgresTarifaServicioRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<TarifaServicioModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = tarifas_servicios::table
            .find(id)
            .first::<TarifaServicioModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_vigente(
        &self,
        tipo_servicio: &str,
        id_servicio: i32,
        fecha: NaiveDate,
    ) -> Result<Option<TarifaServicioModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = tarifas_servicios::table
            .filter(tarifas_servicios::tipo_servicio.eq(tipo_servicio))
            .filter(tarifas_servicios::id_servicio.eq(id_servicio))
            .filter(tarifas_servicios::is_active.eq(true))
            .filter(tarifas_servicios::vigente_desde.le(fecha))
            .filter(
                tarifas_servicios::vigente_hasta.is_null()
                    .or(tarifas_servicios::vigente_hasta.ge(fecha))
            )
            .order(tarifas_servicios::vigente_desde.desc())
            .first::<TarifaServicioModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_by_tipo(
        &self,
        tipo_servicio: &str,
        solo_activas: bool,
    ) -> Result<Vec<TarifaServicioModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = tarifas_servicios::table
            .filter(tarifas_servicios::tipo_servicio.eq(tipo_servicio))
            .into_boxed();
        
        if solo_activas {
            query = query.filter(tarifas_servicios::is_active.eq(true));
        }
        
        let result = query
            .order((tarifas_servicios::tipo_servicio.asc(), tarifas_servicios::vigente_desde.desc()))
            .load::<TarifaServicioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find_all(&self, solo_activas: bool) -> Result<Vec<TarifaServicioModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = tarifas_servicios::table.into_boxed();
        
        if solo_activas {
            query = query.filter(tarifas_servicios::is_active.eq(true));
        }
        
        let result = query
            .order((tarifas_servicios::tipo_servicio.asc(), tarifas_servicios::vigente_desde.desc()))
            .load::<TarifaServicioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewTarifaServicioModel<'_>) -> Result<TarifaServicioModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::insert_into(tarifas_servicios::table)
            .values(&data)
            .get_result::<TarifaServicioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("Tarifa servicio creada: {} - {} ({})", result.id, result.tipo_servicio, result.id_servicio);
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdateTarifaServicioModel) -> Result<TarifaServicioModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(tarifas_servicios::table.find(id))
            .set(&data)
            .get_result::<TarifaServicioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn deactivate(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::update(tarifas_servicios::table.find(id))
            .set(tarifas_servicios::is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
}
