use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{PersonaRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Persona, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{PersonaModel, NewPersonaModel, UpdatePersonaModel},
    schema::personas,
};

pub struct PostgresPersonaRepository { pool: DatabasePool }

impl PostgresPersonaRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl PersonaRepositoryPort for PostgresPersonaRepository {
    #[instrument(skip(self, persona))]
    async fn create(&self, persona: &Persona) -> Result<Persona, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_p: NewPersonaModel = persona.into();
        let result = diesel::insert_into(personas::table).values(&new_p)
            .get_result::<PersonaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Persona creada: {} {} (id: {})", result.nombre, result.apellidos, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Persona>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = personas::table.filter(personas::id.eq(id))
            .first::<PersonaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, persona: &Persona) -> Result<Persona, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let tipo_doc_str = persona.tipo_documento.to_string();
        let changes = UpdatePersonaModel {
            tipo_documento: Some(&tipo_doc_str),
            nro_documento: Some(&persona.nro_documento),
            nombre: Some(&persona.nombre),
            apellidos: Some(&persona.apellidos),
            telefono: Some(persona.telefono.as_deref()),
            correo: Some(persona.correo.as_deref()),
            fecha_nacimiento: Some(persona.fecha_nacimiento),
            updated_by: persona.updated_by,
        };
        let result = diesel::update(personas::table.filter(personas::id.eq(persona.id)))
            .set(&changes).get_result::<PersonaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(personas::table.filter(personas::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete)
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        // Ya hace DELETE real, solo delegamos
        self.delete(id).await
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Persona>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = personas::table.order(personas::apellidos.asc())
            .limit(limit).offset(offset)
            .load::<PersonaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        personas::table.count().get_result::<i64>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Persona>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn find_by_documento(&self, tipo: &str, numero: &str) -> Result<Option<Persona>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = personas::table
            .filter(personas::tipo_documento.eq(tipo))
            .filter(personas::nro_documento.eq(numero))
            .first::<PersonaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<Persona>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = personas::table.filter(personas::correo.eq(email))
            .first::<PersonaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_documento(&self, tipo: &str, numero: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_documento(tipo, numero).await?.is_some())
    }
    
    async fn search(&self, query: &str) -> Result<Vec<Persona>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let pattern = format!("%{}%", query.to_lowercase());
        let results = personas::table
            .filter(
                personas::nombre.ilike(&pattern)
                .or(personas::apellidos.ilike(&pattern))
                .or(personas::nro_documento.ilike(&pattern))
            )
            .order(personas::apellidos.asc())
            .limit(50) // Límite fijo
            .load::<PersonaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}
