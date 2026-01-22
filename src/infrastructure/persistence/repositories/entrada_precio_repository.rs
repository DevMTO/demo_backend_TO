use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{instrument, debug};

use crate::application::ports::EntradaPrecioRepositoryPort;
use crate::domain::entities::EntradaPrecio;
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;
use crate::infrastructure::persistence::models::{
    EntradaPrecioModel, NewEntradaPrecioModel,
};
use crate::infrastructure::persistence::schema::entrada_precios;

pub struct PostgresEntradaPrecioRepository {
    pool: DatabasePool,
}

impl PostgresEntradaPrecioRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntradaPrecioRepositoryPort for PostgresEntradaPrecioRepository {
    #[instrument(skip(self, precio))]
    async fn create(&self, precio: &EntradaPrecio) -> Result<EntradaPrecio, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_model = NewEntradaPrecioModel::from(precio);
        
        let result = diesel::insert_into(entrada_precios::table)
            .values(&new_model)
            .returning(EntradaPrecioModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al crear precio: {e}")))?;
        
        debug!("Precio de entrada creado: ID {}", result.id);
        Ok(result.into())
    }
    
    #[instrument(skip(self, precios))]
    async fn create_batch(&self, precios: &[EntradaPrecio]) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        if precios.is_empty() {
            return Ok(vec![]);
        }
        
        let mut conn = self.pool.get_connection().await?;
        let new_models: Vec<NewEntradaPrecioModel> = precios
            .iter()
            .map(NewEntradaPrecioModel::from)
            .collect();
        
        let results = diesel::insert_into(entrada_precios::table)
            .values(&new_models)
            .returning(EntradaPrecioModel::as_returning())
            .get_results(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al crear precios en batch: {e}")))?;
        
        debug!("{} precios de entrada creados en batch", results.len());
        Ok(results.into_iter().map(EntradaPrecio::from).collect())
    }
    
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<EntradaPrecio>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = entrada_precios::table
            .find(id)
            .first::<EntradaPrecioModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(format!("Error al buscar precio: {e}")))?;
        
        Ok(result.map(EntradaPrecio::from))
    }
    
    #[instrument(skip(self))]
    async fn find_by_entrada(&self, id_entrada: i32) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = entrada_precios::table
            .filter(entrada_precios::id_entrada.eq(id_entrada))
            .order((entrada_precios::tipo_precio.asc(), entrada_precios::edad_minima.asc()))
            .load::<EntradaPrecioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al buscar precios: {e}")))?;
        
        Ok(results.into_iter().map(EntradaPrecio::from).collect())
    }
    
    #[instrument(skip(self))]
    async fn find_by_entrada_and_tipo(&self, id_entrada: i32, tipo_precio: &str) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let results = entrada_precios::table
            .filter(entrada_precios::id_entrada.eq(id_entrada))
            .filter(entrada_precios::tipo_precio.eq(tipo_precio))
            .order(entrada_precios::edad_minima.asc())
            .load::<EntradaPrecioModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al buscar precios por tipo: {e}")))?;
        
        Ok(results.into_iter().map(EntradaPrecio::from).collect())
    }
    
    #[instrument(skip(self))]
    async fn find_precio_for_edad(&self, id_entrada: i32, tipo_precio: &str, edad: i32) -> Result<Option<EntradaPrecio>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Buscar precio donde edad_minima <= edad AND (edad_maxima IS NULL OR edad_maxima >= edad)
        let result = entrada_precios::table
            .filter(entrada_precios::id_entrada.eq(id_entrada))
            .filter(entrada_precios::tipo_precio.eq(tipo_precio))
            .filter(entrada_precios::edad_minima.le(edad))
            .filter(
                entrada_precios::edad_maxima.is_null()
                    .or(entrada_precios::edad_maxima.ge(edad))
            )
            .first::<EntradaPrecioModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(format!("Error al buscar precio por edad: {e}")))?;
        
        Ok(result.map(EntradaPrecio::from))
    }
    
    #[instrument(skip(self, precio))]
    async fn update(&self, precio: &EntradaPrecio) -> Result<EntradaPrecio, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(entrada_precios::table.find(precio.id))
            .set((
                entrada_precios::tipo_precio.eq(&precio.tipo_precio),
                entrada_precios::edad_minima.eq(precio.edad_minima),
                entrada_precios::edad_maxima.eq(precio.edad_maxima),
                entrada_precios::precio.eq(&precio.precio),
                entrada_precios::descripcion.eq(&precio.descripcion),
                entrada_precios::updated_by.eq(precio.updated_by),
            ))
            .returning(EntradaPrecioModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al actualizar precio: {e}")))?;
        
        debug!("Precio de entrada actualizado: ID {}", result.id);
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let deleted = diesel::delete(entrada_precios::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error al eliminar precio: {e}")))?;
        
        debug!("🗑️ Precio de entrada eliminado: ID {}", id);
        Ok(deleted > 0)
    }
    
    #[instrument(skip(self))]
    async fn delete_by_entrada(&self, id_entrada: i32) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let deleted = diesel::delete(
            entrada_precios::table.filter(entrada_precios::id_entrada.eq(id_entrada))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(format!("Error al eliminar precios: {e}")))?;
        
        debug!("🗑️ {} precios eliminados para entrada {}", deleted, id_entrada);
        Ok(deleted as i64)
    }
    
    #[instrument(skip(self, precios))]
    async fn replace_all(&self, id_entrada: i32, precios: &[EntradaPrecio]) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        // Primero eliminar todos los precios existentes
        self.delete_by_entrada(id_entrada).await?;
        
        // Luego crear los nuevos precios
        self.create_batch(precios).await
    }
}
