use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{FileRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::File, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{FileModel, NewFileModel, UpdateFileModel},
    schema::files,
};

pub struct PostgresFileRepository {
    pool: DatabasePool,
}

impl PostgresFileRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepositoryPort for PostgresFileRepository {
    #[instrument(skip(self, file))]
    async fn create(&self, file: &File) -> Result<File, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_f: NewFileModel = file.into();
        
        let result = diesel::insert_into(files::table)
            .values(&new_f)
            .returning(FileModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        info!("File creado: agencia={} (id: {})", result.id_entidad, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = files::table
            .filter(files::id.eq(id))
            .select(FileModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, file: &File) -> Result<File, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let changes = UpdateFileModel {
            // id_tour eliminado - ahora en file_tours
            id_entidad: Some(file.id_entidad),
            entidad: Some(file.entidad.as_deref()),
            fecha_inicio: Some(file.fecha_inicio),
            fecha_fin: Some(file.fecha_fin),
            // lugar_recojo, hora_recojo, turno_tour ahora están en file_tours
            notas: Some(file.notas.as_deref()),
            status: Some(&file.status),
            monto_total: Some(file.monto_total.clone()),
            monto_pagado: Some(file.monto_pagado.clone()),
            is_active: Some(file.is_active),
            nro_pasajeros: Some(file.nro_pasajeros),
            file_code: Some(file.file_code.as_deref()),
            deadline_confirmacion: Some(file.deadline_confirmacion),
            updated_by: file.updated_by,
        };
        
        let result = diesel::update(files::table.filter(files::id.eq(file.id)))
            .set(&changes)
            .returning(FileModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::delete(files::table.filter(files::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(files::table.filter(files::id.eq(id)))
            .set((files::is_active.eq(false), files::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(files::table.filter(files::id.eq(id)))
            .set((files::is_active.eq(true), files::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete)
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        self.delete(id).await
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = files::table
            .order(files::fecha_inicio.desc())
            .limit(limit)
            .offset(offset)
            .select(FileModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        files::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<File>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn find_by_entidad(&self, id_entidad: i32) -> Result<Vec<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = files::table
            .filter(files::id_entidad.eq(id_entidad))
            .order(files::fecha_inicio.desc())
            .select(FileModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_date_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = files::table
            .filter(files::fecha_inicio.ge(from))
            .filter(files::fecha_fin.le(to))
            .order(files::fecha_inicio.asc())
            .select(FileModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_upcoming(&self) -> Result<Vec<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let today = Utc::now().date_naive();
        
        let results = files::table
            .filter(files::fecha_inicio.ge(today))
            .filter(files::status.ne("cancelado"))
            .filter(files::status.ne("anulado"))
            .filter(files::status.ne("no_show"))
            .order(files::fecha_inicio.asc())
            .limit(50)
            .select(FileModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_pending_payment(&self) -> Result<Vec<File>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = files::table
            .filter(files::status.eq("pendiente_pago"))
            .order(files::fecha_inicio.desc())
            .select(FileModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Actualiza solo el estado de un File
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let affected = diesel::update(files::table.filter(files::id.eq(id)))
            .set(files::status.eq(status))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(affected > 0)
    }

}
