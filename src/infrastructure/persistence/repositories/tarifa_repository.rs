use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::info;

use crate::application::ports::TarifaRepositoryPort;
use crate::domain::{entities::Tarifa, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{TarifaModel, NewTarifaModel, UpdateTarifaModel},
    schema::tarifas,
};

pub struct PostgresTarifaRepository {
    pool: DatabasePool,
}

impl PostgresTarifaRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TarifaRepositoryPort for PostgresTarifaRepository {
    async fn create(&self, tarifa: &Tarifa) -> Result<Tarifa, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new: NewTarifaModel = tarifa.into();

        let result = diesel::insert_into(tarifas::table)
            .values(&new)
            .returning(TarifaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        info!("Tarifa creada: tour={} tipo={} (id: {})", result.id_tour, result.tipo_entidad, result.id);
        Ok(result.into())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Tarifa>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let result = tarifas::table
            .filter(tarifas::id.eq(id))
            .select(TarifaModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_tour(&self, id_tour: i32) -> Result<Vec<Tarifa>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let results = tarifas::table
            .filter(tarifas::id_tour.eq(id_tour))
            .order(tarifas::tipo_entidad.asc())
            .select(TarifaModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_tour_and_tipo(&self, id_tour: i32, tipo_entidad: &str) -> Result<Option<Tarifa>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let result = tarifas::table
            .filter(tarifas::id_tour.eq(id_tour))
            .filter(tarifas::tipo_entidad.eq(tipo_entidad))
            .select(TarifaModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, tarifa: &Tarifa) -> Result<Tarifa, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let changes = UpdateTarifaModel {
            tipo_entidad: Some(&tarifa.tipo_entidad),
            precio: Some(tarifa.precio.clone()),
            descripcion: Some(tarifa.descripcion.as_deref()),
            is_active: Some(tarifa.is_active),
            updated_by: tarifa.updated_by,
        };

        let result = diesel::update(tarifas::table.filter(tarifas::id.eq(tarifa.id)))
            .set(&changes)
            .returning(TarifaModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(result.into())
    }

    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let affected = diesel::delete(tarifas::table.filter(tarifas::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(affected > 0)
    }

    async fn delete_by_tour(&self, id_tour: i32) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;

        let affected = diesel::delete(tarifas::table.filter(tarifas::id_tour.eq(id_tour)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(affected as i64)
    }
}
