use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, info, instrument};

use crate::application::ports::{AgenciaRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Agencia, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{AgenciaModel, NewAgenciaModel, UpdateAgenciaModel},
    schema::agencias,
};

pub struct PostgresAgenciaRepository {
    pool: DatabasePool,
}

impl PostgresAgenciaRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgenciaRepositoryPort for PostgresAgenciaRepository {
    #[instrument(skip(self, agencia))]
    async fn create(&self, agencia: &Agencia) -> Result<Agencia, ApplicationError> {
        debug!("📝 Creando agencia: {}", agencia.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_agencia: NewAgenciaModel = agencia.into();
        
        let result = diesel::insert_into(agencias::table)
            .values(&new_agencia)
            .get_result::<AgenciaModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("❌ Error al crear agencia: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("✅ Agencia creada: {} (id: {})", result.nombre, result.id);
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<Agencia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = agencias::table
            .filter(agencias::id.eq(id))
            .first::<AgenciaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self, agencia))]
    async fn update(&self, agencia: &Agencia) -> Result<Agencia, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateAgenciaModel {
            nombre: Some(&agencia.nombre),
            ruc: Some(&agencia.ruc),
            telefono: Some(agencia.telefono.as_deref()),
            correo: Some(agencia.correo.as_deref()),
            direccion: Some(agencia.direccion.as_deref()),
            paleta_colores: Some(agencia.paleta_colores.clone()),
            media: Some(agencia.media.clone()),
            encargado: Some(agencia.encargado),
            is_active: Some(agencia.is_active),
            updated_by: agencia.updated_by,
        };
        let result = diesel::update(agencias::table.filter(agencias::id.eq(agencia.id)))
            .set(&changes)
            .get_result::<AgenciaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(agencias::table.filter(agencias::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Agencia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = agencias::table
            .filter(agencias::is_active.eq(true))
            .order(agencias::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<AgenciaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        agencias::table
            .filter(agencias::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Agencia>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(agencias::table.filter(agencias::id.eq(id)))
            .set((agencias::is_active.eq(false), agencias::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(agencias::table.filter(agencias::id.eq(id)))
            .set((agencias::is_active.eq(true), agencias::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn find_by_ruc(&self, ruc: &str) -> Result<Option<Agencia>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = agencias::table
            .filter(agencias::ruc.eq(ruc))
            .first::<AgenciaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_ruc(&self, ruc: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_ruc(ruc).await?.is_some())
    }
    
    #[instrument(skip(self))]
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<crate::application::dtos::AgenciaListItemDto>, i64), ApplicationError> {
        use crate::infrastructure::persistence::schema::personas;
        
        debug!("📋 Listando agencias con encargado (limit: {}, offset: {})", limit, offset);
        let mut conn = self.pool.get_connection().await?;
        
        let total: i64 = agencias::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results: Vec<(AgenciaModel, Option<(String, String)>)> = agencias::table
            .left_join(personas::table.on(agencias::encargado.eq(personas::id.nullable())))
            .select((
                AgenciaModel::as_select(),
                (personas::nombre, personas::apellidos).nullable(),
            ))
            .order(agencias::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<crate::application::dtos::AgenciaListItemDto> = results
            .into_iter()
            .map(|(agencia, persona_data)| {
                let encargado_nombre = persona_data.map(|(nombre, apellidos)| {
                    format!("{} {}", nombre, apellidos)
                });
                
                crate::application::dtos::AgenciaListItemDto {
                    id: agencia.id,
                    nombre: agencia.nombre,
                    ruc: agencia.ruc,
                    telefono: agencia.telefono,
                    correo: agencia.correo,
                    direccion: agencia.direccion,
                    paleta_colores: agencia.paleta_colores,
                    media: agencia.media,
                    encargado: agencia.encargado,
                    encargado_nombre,
                    is_active: agencia.is_active,
                    created_at: agencia.created_at,
                    updated_at: agencia.updated_at,
                }
            })
            .collect();
        
        info!("✅ Listadas {} agencias de {} total", items.len(), total);
        Ok((items, total))
    }
}
