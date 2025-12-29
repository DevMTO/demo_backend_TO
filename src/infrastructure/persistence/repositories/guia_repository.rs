use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{GuiaRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Guia, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{GuiaModel, NewGuiaModel, UpdateGuiaModel},
    schema::guias,
};

pub struct PostgresGuiaRepository { pool: DatabasePool }

impl PostgresGuiaRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl GuiaRepositoryPort for PostgresGuiaRepository {
    #[instrument(skip(self, guia))]
    async fn create(&self, guia: &Guia) -> Result<Guia, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_g: NewGuiaModel = guia.into();
        let result = diesel::insert_into(guias::table).values(&new_g)
            .get_result::<GuiaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("✅ Guia creado: {} (id: {})", result.nro_carnet, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Guia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = guias::table.filter(guias::id.eq(id))
            .first::<GuiaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, guia: &Guia) -> Result<Guia, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let status_str = guia.status.to_string();
        let changes = UpdateGuiaModel {
            nro_carnet: Some(&guia.nro_carnet),
            idiomas: Some(guia.idiomas.clone()),
            especialidades: Some(guia.especialidades.clone()),
            status: Some(&status_str),
            updated_by: guia.updated_by,
        };
        let result = diesel::update(guias::table.filter(guias::id.eq(guia.id)))
            .set(&changes).get_result::<GuiaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(guias::table.filter(guias::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Guia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = guias::table.order(guias::nro_carnet.asc()).limit(limit).offset(offset)
            .load::<GuiaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        guias::table.count().get_result::<i64>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Guia>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn find_by_carnet(&self, nro_carnet: &str) -> Result<Option<Guia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = guias::table.filter(guias::nro_carnet.eq(nro_carnet))
            .first::<GuiaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_carnet(&self, nro_carnet: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_carnet(nro_carnet).await?.is_some())
    }
    
    async fn list_available(&self) -> Result<Vec<Guia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = guias::table.filter(guias::status.eq("disponible"))
            .load::<GuiaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_idioma(&self, _idioma: &str) -> Result<Vec<Guia>, ApplicationError> {
        // TODO: Implement JSONB search for idiomas
        self.list(100, 0).await
    }
    
    async fn find_by_especialidad(&self, _especialidad: &str) -> Result<Vec<Guia>, ApplicationError> {
        // TODO: Implement JSONB search for especialidades
        self.list(100, 0).await
    }
}
