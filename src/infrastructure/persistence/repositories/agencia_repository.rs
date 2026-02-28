use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, info, instrument};

use crate::application::ports::{AgenciaRepositoryPort, PaginationOptions, PaginatedResult, CachePort, entity_names};
use crate::domain::{entities::Agencia, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{AgenciaModel, NewAgenciaModel, UpdateAgenciaModel},
    schema::agencias,
};

pub struct PostgresAgenciaRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresAgenciaRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        Self { pool, cache }
    }

    fn list_cache_key(limit: i64, offset: i64) -> String {
        format!("list:{}:{}", limit, offset)
    }

    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::AGENCIAS).await;
        info!("[CACHE INVALIDATED] agencias");
    }
}

#[async_trait]
impl AgenciaRepositoryPort for PostgresAgenciaRepository {
    #[instrument(skip(self, agencia))]
    async fn create(&self, agencia: &Agencia) -> Result<Agencia, ApplicationError> {
        debug!("Creando agencia: {}", agencia.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_agencia: NewAgenciaModel = agencia.into();
        
        let result = diesel::insert_into(agencias::table)
            .values(&new_agencia)
            .get_result::<AgenciaModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al crear agencia: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("Agencia creada: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<Agencia>, ApplicationError> {
        let start = Instant::now();
        
        // Check cache
        if let Some(cached) = self.cache.get_detail(entity_names::AGENCIAS, id).await {
            if let Ok(agencia) = serde_json::from_str::<Agencia>(&cached) {
                info!("[CACHE HIT] agencia #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(agencia));
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        let result = agencias::table
            .filter(agencias::id.eq(id))
            .first::<AgenciaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if let Some(ref model) = result {
            let agencia: Agencia = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&agencia) {
                self.cache.set_detail(entity_names::AGENCIAS, id, serialized).await;
            }
            info!("[CACHE MISS → DB] agencia #{} | {}ms", id, start.elapsed().as_millis());
            return Ok(Some(agencia));
        }
        
        Ok(None)
    }
    
    #[instrument(skip(self))]
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<Agencia>, ApplicationError> {
        debug!("Buscando agencia por encargado (persona_id: {})", persona_id);
        let mut conn = self.pool.get_connection().await?;
        let result = agencias::table
            .filter(agencias::encargado.eq(Some(persona_id)))
            .filter(agencias::is_active.eq(true))
            .first::<AgenciaModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if let Some(ref agencia) = result {
            info!("Encontrada agencia '{}' (id: {}) para encargado {}", agencia.nombre, agencia.id, persona_id);
        } else {
            debug!("ℹ️ No se encontró agencia para encargado {}", persona_id);
        }
        
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
            pago_anticipado: Some(agencia.pago_anticipado),
            tipo_vencimiento: Some(agencia.tipo_vencimiento.as_deref()),
            updated_by: agencia.updated_by,
        };
        let result = diesel::update(agencias::table.filter(agencias::id.eq(agencia.id)))
            .set(&changes)
            .get_result::<AgenciaModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(agencias::table.filter(agencias::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    #[instrument(skip(self))]
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        debug!("Hard delete de agencia: {}", id);
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(agencias::table.filter(agencias::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
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
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(agencias::table.filter(agencias::id.eq(id)))
            .set((agencias::is_active.eq(true), agencias::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
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
        
        let start = Instant::now();
        let cache_key = Self::list_cache_key(limit, offset);
        
        // Check cache
        if let Some(cached) = self.cache.get_list(entity_names::AGENCIAS, &cache_key).await {
            if let Ok(response) = serde_json::from_str::<(Vec<crate::application::dtos::AgenciaListItemDto>, i64)>(&cached) {
                info!("[CACHE HIT] agencias list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(response);
            }
        }
        
        debug!("Listando agencias con encargado (limit: {}, offset: {})", limit, offset);
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
                    pago_anticipado: agencia.pago_anticipado,
                    tipo_vencimiento: agencia.tipo_vencimiento,
                    created_at: agencia.created_at,
                    updated_at: agencia.updated_at,
                }
            })
            .collect();
        
        let response = (items, total);
        
        // Store in cache
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::AGENCIAS, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS → DB] agencias list '{}' ({} items, total: {}) | {}ms", cache_key, response.0.len(), total, start.elapsed().as_millis());
        Ok(response)
    }
}
