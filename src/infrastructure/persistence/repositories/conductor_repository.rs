use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::ConductorRepositoryPort;
use crate::application::dtos::ConductorListItemDto;
use crate::domain::{entities::Conductor, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{ConductorModel, NewConductorModel, UpdateConductorModel},
    schema::{conductores, personas, transportes},
};

pub struct PostgresConductorRepository { pool: DatabasePool }

impl PostgresConductorRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl ConductorRepositoryPort for PostgresConductorRepository {
    #[instrument(skip(self, conductor))]
    async fn create(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_c: NewConductorModel = conductor.into();
        let result = diesel::insert_into(conductores::table).values(&new_c)
            .get_result::<ConductorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Conductor creado: {} (id: {})", result.nro_brevete, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Conductor>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = conductores::table.filter(conductores::id.eq(id))
            .first::<ConductorModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, conductor: &Conductor) -> Result<Conductor, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let status_str = conductor.status.to_string();
        let changes = UpdateConductorModel {
            id_transporte: Some(conductor.id_transporte),
            nro_brevete: Some(&conductor.nro_brevete),
            tiene_soat: Some(conductor.tiene_soat),
            status: Some(&status_str),
            is_active: Some(conductor.is_active),
            updated_by: conductor.updated_by,
        };
        let result = diesel::update(conductores::table.filter(conductores::id.eq(conductor.id)))
            .set(&changes).get_result::<ConductorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(conductores::table.filter(conductores::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete)
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        self.delete(id).await
    }
    
    #[instrument(skip(self))]
    async fn list_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<ConductorListItemDto>, i64), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let total: i64 = conductores::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results: Vec<(ConductorModel, Option<(String, String, String)>, Option<String>)> = conductores::table
            .left_join(personas::table.on(conductores::id_persona.eq(personas::id)))
            .left_join(transportes::table.on(conductores::id_transporte.eq(transportes::id.nullable())))
            .select((
                ConductorModel::as_select(),
                (personas::nombre, personas::apellidos, personas::nro_documento).nullable(),
                transportes::nombre.nullable(),
            ))
            .order(conductores::nro_brevete.asc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<ConductorListItemDto> = results
            .into_iter()
            .map(|(conductor, persona_data, transporte_nombre)| {
                let (persona_nombre, persona_documento) = persona_data
                    .map(|(nombre, apellidos, doc)| {
                        (Some(format!("{} {}", nombre, apellidos)), Some(doc))
                    })
                    .unwrap_or((None, None));
                
                ConductorListItemDto {
                    id: conductor.id,
                    id_persona: conductor.id_persona,
                    persona_nombre,
                    persona_documento,
                    id_transporte: conductor.id_transporte,
                    transporte_nombre,
                    nro_brevete: conductor.nro_brevete,
                    tiene_soat: conductor.tiene_soat,
                    status: conductor.status,
                    is_active: conductor.is_active,
                    created_at: conductor.created_at,
                    updated_at: conductor.updated_at,
                }
            })
            .collect();
        
        info!("Listados {} conductores de {} total", items.len(), total);
        Ok((items, total))
    }
    
    async fn find_by_brevete(&self, nro_brevete: &str) -> Result<Option<Conductor>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = conductores::table.filter(conductores::nro_brevete.eq(nro_brevete))
            .first::<ConductorModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_brevete(&self, nro_brevete: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_brevete(nro_brevete).await?.is_some())
    }
    
    async fn find_by_transporte(&self, id_transporte: i32) -> Result<Vec<Conductor>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = conductores::table.filter(conductores::id_transporte.eq(id_transporte))
            .load::<ConductorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn list_available(&self) -> Result<Vec<Conductor>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = conductores::table.filter(conductores::status.eq("disponible"))
            .load::<ConductorModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(conductores::table.filter(conductores::id.eq(id)))
            .set(conductores::status.eq(status))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Conductor {} status actualizado a: {}", id, status);
        Ok(affected > 0)
    }
}
