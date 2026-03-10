use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Tarifa;

#[async_trait]
pub trait TarifaRepositoryPort: Send + Sync {
    async fn create(&self, tarifa: &Tarifa) -> Result<Tarifa, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Tarifa>, ApplicationError>;
    async fn find_by_tour(&self, id_tour: i32) -> Result<Vec<Tarifa>, ApplicationError>;
    async fn find_by_tour_and_tipo(&self, id_tour: i32, tipo_entidad: &str) -> Result<Option<Tarifa>, ApplicationError>;
    async fn update(&self, tarifa: &Tarifa) -> Result<Tarifa, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn delete_by_tour(&self, id_tour: i32) -> Result<i64, ApplicationError>;
}
