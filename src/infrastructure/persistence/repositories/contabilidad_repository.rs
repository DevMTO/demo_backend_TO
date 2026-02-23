use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::contabilidad_repository::{
    PagoFileRepositoryPort, PagoProveedorRepositoryPort,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;
use crate::infrastructure::persistence::models::{
    PagoFileModel, PagoProveedorModel,
    NewPagoFileModel, NewPagoProveedorModel,
    UpdatePagoFileModel, UpdatePagoProveedorModel,
};
use crate::infrastructure::persistence::schema::{pagos_files, pagos_proveedores};

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
        pagos_files::table.find(id).first::<PagoFileModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_all_by_file(&self, id_file: i32) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        pagos_files::table.filter(pagos_files::id_file.eq(id_file))
            .order(pagos_files::created_at.asc())
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_by_agencia(&self, id_agencia: i32, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        pagos_files::table.filter(pagos_files::id_agencia.eq(id_agencia))
            .order(pagos_files::created_at.desc()).limit(limit).offset(offset)
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_filtered(&self, id_agencia: Option<i32>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_files::table.into_boxed();
        if let Some(a) = id_agencia { query = query.filter(pagos_files::id_agencia.eq(a)); }
        if let Some(e) = estado { query = query.filter(pagos_files::estado.eq(e)); }
        if let Some(d) = fecha_desde { query = query.filter(pagos_files::fecha_vencimiento.ge(d)); }
        if let Some(h) = fecha_hasta { query = query.filter(pagos_files::fecha_vencimiento.le(h)); }
        query.order(pagos_files::created_at.desc()).limit(limit).offset(offset)
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn count_filtered(&self, id_agencia: Option<i32>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_files::table.into_boxed();
        if let Some(a) = id_agencia { query = query.filter(pagos_files::id_agencia.eq(a)); }
        if let Some(e) = estado { query = query.filter(pagos_files::estado.eq(e)); }
        if let Some(d) = fecha_desde { query = query.filter(pagos_files::fecha_vencimiento.ge(d)); }
        if let Some(h) = fecha_hasta { query = query.filter(pagos_files::fecha_vencimiento.le(h)); }
        query.count().get_result::<i64>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewPagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = diesel::insert_into(pagos_files::table).values(&data)
            .get_result::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Pago file creado: {} - file {}", result.id, result.id_file);
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdatePagoFileModel<'_>) -> Result<PagoFileModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        diesel::update(pagos_files::table.find(id)).set(&data)
            .get_result::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_by_agencia_tipos(&self, id_agencia: i32, tipos: &[&str], limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let tipos_owned: Vec<String> = tipos.iter().map(|t| t.to_string()).collect();
        pagos_files::table
            .filter(pagos_files::id_agencia.eq(id_agencia))
            .filter(pagos_files::tipo_registro.eq_any(&tipos_owned))
            .order(pagos_files::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
}

pub struct PostgresPagoProveedorRepository {
    pool: DatabasePool,
}

impl PostgresPagoProveedorRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl PagoProveedorRepositoryPort for PostgresPagoProveedorRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        pagos_proveedores::table.find(id).first::<PagoProveedorModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_filtered(&self, tipo_proveedor: Option<&str>, estado: Option<&str>, fecha_desde: Option<DateTime<Utc>>, fecha_hasta: Option<DateTime<Utc>>, limit: i64, offset: i64) -> Result<Vec<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_proveedores::table.into_boxed();
        if let Some(tp) = tipo_proveedor { query = query.filter(pagos_proveedores::tipo_proveedor.eq(tp)); }
        if let Some(e) = estado { query = query.filter(pagos_proveedores::estado.eq(e)); }
        if let Some(d) = fecha_desde { query = query.filter(pagos_proveedores::created_at.ge(d)); }
        if let Some(h) = fecha_hasta { query = query.filter(pagos_proveedores::created_at.le(h)); }
        query.order(pagos_proveedores::created_at.desc()).limit(limit).offset(offset)
            .load::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn count_filtered(&self, tipo_proveedor: Option<&str>, estado: Option<&str>, fecha_desde: Option<DateTime<Utc>>, fecha_hasta: Option<DateTime<Utc>>) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_proveedores::table.into_boxed();
        if let Some(tp) = tipo_proveedor { query = query.filter(pagos_proveedores::tipo_proveedor.eq(tp)); }
        if let Some(e) = estado { query = query.filter(pagos_proveedores::estado.eq(e)); }
        if let Some(d) = fecha_desde { query = query.filter(pagos_proveedores::created_at.ge(d)); }
        if let Some(h) = fecha_hasta { query = query.filter(pagos_proveedores::created_at.le(h)); }
        query.count().get_result::<i64>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self, data))]
    async fn create(&self, data: NewPagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = diesel::insert_into(pagos_proveedores::table).values(&data)
            .get_result::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Pago proveedor creado: {} - {}", result.id, result.tipo_proveedor);
        Ok(result)
    }

    #[instrument(skip(self, data))]
    async fn update(&self, id: i32, data: UpdatePagoProveedorModel<'_>) -> Result<PagoProveedorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        diesel::update(pagos_proveedores::table.find(id)).set(&data)
            .get_result::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_by_file_relation(&self, tipo_proveedor: &str, id_file_vehiculo: Option<i32>, id_file_restaurante: Option<i32>, id_file_guia: Option<i32>) -> Result<Option<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_proveedores::table.filter(pagos_proveedores::tipo_proveedor.eq(tipo_proveedor)).into_boxed();
        if let Some(id) = id_file_vehiculo { query = query.filter(pagos_proveedores::id_file_vehiculo.eq(id)); }
        if let Some(id) = id_file_restaurante { query = query.filter(pagos_proveedores::id_file_restaurante.eq(id)); }
        if let Some(id) = id_file_guia { query = query.filter(pagos_proveedores::id_file_guia.eq(id)); }
        query.first::<PagoProveedorModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
}