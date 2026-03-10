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
use crate::infrastructure::persistence::schema::{file_tours, pagos_files, pagos_proveedores};

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
    async fn find_by_entidad(&self, id_entidad: i32, entidad: Option<&str>, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_files::table
            .filter(pagos_files::id_entidad.eq(id_entidad))
            .into_boxed();
        if let Some(ent) = entidad {
            query = query.filter(pagos_files::entidad.eq(ent));
        }
        query
            .order(pagos_files::created_at.desc()).limit(limit).offset(offset)
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_filtered(&self, id_entidad: Option<i32>, entidad: Option<&str>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>, limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_files::table.into_boxed();
        if let Some(a) = id_entidad { query = query.filter(pagos_files::id_entidad.eq(a)); }
        if let Some(ent) = entidad { query = query.filter(pagos_files::entidad.eq(ent)); }
        if let Some(e) = estado { query = query.filter(pagos_files::estado.eq(e)); }
        if let Some(d) = fecha_desde { query = query.filter(pagos_files::fecha_vencimiento.ge(d)); }
        if let Some(h) = fecha_hasta { query = query.filter(pagos_files::fecha_vencimiento.le(h)); }
        query.order(pagos_files::created_at.desc()).limit(limit).offset(offset)
            .load::<PagoFileModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn count_filtered(&self, id_entidad: Option<i32>, entidad: Option<&str>, estado: Option<&str>, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_files::table.into_boxed();
        if let Some(a) = id_entidad { query = query.filter(pagos_files::id_entidad.eq(a)); }
        if let Some(ent) = entidad { query = query.filter(pagos_files::entidad.eq(ent)); }
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
    async fn find_by_entidad_tipos(&self, id_entidad: i32, entidad: Option<&str>, tipos: &[&str], limit: i64, offset: i64) -> Result<Vec<PagoFileModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let tipos_owned: Vec<String> = tipos.iter().map(|t| t.to_string()).collect();
        let mut query = pagos_files::table
            .filter(pagos_files::id_entidad.eq(id_entidad))
            .filter(pagos_files::tipo_registro.eq_any(&tipos_owned))
            .into_boxed();
        
        if let Some(ent) = entidad {
            query = query.filter(pagos_files::entidad.eq(ent));
        }
        
        query
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
    async fn find_by_file_relation(&self, tipo_proveedor: &str, id_file_vehiculo: Option<i32>, id_file_restaurante: Option<i32>, id_file_guia: Option<i32>, id_file_entrada: Option<i32>) -> Result<Option<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let mut query = pagos_proveedores::table.filter(pagos_proveedores::tipo_proveedor.eq(tipo_proveedor)).into_boxed();
        if let Some(id) = id_file_vehiculo { query = query.filter(pagos_proveedores::id_file_vehiculo.eq(id)); }
        if let Some(id) = id_file_restaurante { query = query.filter(pagos_proveedores::id_file_restaurante.eq(id)); }
        if let Some(id) = id_file_guia { query = query.filter(pagos_proveedores::id_file_guia.eq(id)); }
        if let Some(id) = id_file_entrada { query = query.filter(pagos_proveedores::id_file_entrada.eq(id)); }
        query.first::<PagoProveedorModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_by_file_tour(&self, id_file_tour: i32) -> Result<Vec<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        pagos_proveedores::table
            .filter(pagos_proveedores::id_file_tour.eq(id_file_tour))
            .load::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn find_by_tipo_tour_fecha(&self, tipo_proveedor: &str, provider_id: i32, tour_id: i32, fecha_tour: NaiveDate, exclude_id: Option<i32>) -> Result<Vec<PagoProveedorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let file_tour_ids: Vec<i32> = file_tours::table
            .filter(file_tours::id_tour.eq(tour_id))
            .filter(file_tours::fecha_tour.eq(fecha_tour))
            .select(file_tours::id)
            .load::<i32>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if file_tour_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut query = pagos_proveedores::table
            .filter(pagos_proveedores::tipo_proveedor.eq(tipo_proveedor))
            .filter(pagos_proveedores::id_file_tour.eq_any(file_tour_ids))
            .into_boxed();
        
        match tipo_proveedor {
            "transporte" => {
                query = query.filter(pagos_proveedores::id_transporte.eq(provider_id));
            },
            "restaurante" => {
                query = query.filter(pagos_proveedores::id_restaurante.eq(provider_id));
            },
            "guia" => {
                query = query.filter(pagos_proveedores::id_guia.eq(provider_id));
            },
            "entrada" => {
                query = query.filter(pagos_proveedores::id_entrada.eq(provider_id));
            },
            _ => {},
        }
        
        if let Some(id) = exclude_id {
            query = query.filter(pagos_proveedores::id.ne(id));
        }
        
        query.load::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn update_status(&self, id: i32, estado: &str) -> Result<PagoProveedorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let data = UpdatePagoProveedorModel { estado: Some(estado), ..Default::default() };
        diesel::update(pagos_proveedores::table.find(id)).set(&data)
            .get_result::<PagoProveedorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
}
