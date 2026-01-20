use async_trait::async_trait;

use crate::domain::entities::EntradaPrecio;
use crate::domain::errors::ApplicationError;

/// Port para el repositorio de EntradaPrecio
/// Sigue el patrón de arquitectura hexagonal
#[async_trait]
pub trait EntradaPrecioRepositoryPort: Send + Sync {
    /// Crear un nuevo precio de entrada
    async fn create(&self, precio: &EntradaPrecio) -> Result<EntradaPrecio, ApplicationError>;
    
    /// Crear múltiples precios de entrada en batch
    async fn create_batch(&self, precios: &[EntradaPrecio]) -> Result<Vec<EntradaPrecio>, ApplicationError>;
    
    /// Obtener precio por ID
    async fn find_by_id(&self, id: i32) -> Result<Option<EntradaPrecio>, ApplicationError>;
    
    /// Obtener todos los precios de una entrada
    async fn find_by_entrada(&self, id_entrada: i32) -> Result<Vec<EntradaPrecio>, ApplicationError>;
    
    /// Obtener precios por entrada y tipo (general, nacional, extranjero)
    async fn find_by_entrada_and_tipo(&self, id_entrada: i32, tipo_precio: &str) -> Result<Vec<EntradaPrecio>, ApplicationError>;
    
    /// Obtener precio aplicable para una entrada, tipo y edad específica
    async fn find_precio_for_edad(&self, id_entrada: i32, tipo_precio: &str, edad: i32) -> Result<Option<EntradaPrecio>, ApplicationError>;
    
    /// Actualizar un precio de entrada
    async fn update(&self, precio: &EntradaPrecio) -> Result<EntradaPrecio, ApplicationError>;
    
    /// Eliminar un precio de entrada
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    
    /// Eliminar todos los precios de una entrada
    async fn delete_by_entrada(&self, id_entrada: i32) -> Result<i64, ApplicationError>;
    
    /// Reemplazar todos los precios de una entrada (delete all + create batch)
    async fn replace_all(&self, id_entrada: i32, precios: &[EntradaPrecio]) -> Result<Vec<EntradaPrecio>, ApplicationError>;
}
