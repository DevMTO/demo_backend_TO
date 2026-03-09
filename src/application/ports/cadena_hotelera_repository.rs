use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::CadenaHotelera;
use crate::application::dtos::CadenaHoteleraListItemDto;

#[async_trait]
pub trait CadenaHoteleraRepositoryPort: Send + Sync {
    async fn create(&self, cadena: &CadenaHotelera) -> Result<CadenaHotelera, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<CadenaHotelera>, ApplicationError>;
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<CadenaHotelera>, ApplicationError>;
    async fn update(&self, cadena: &CadenaHotelera) -> Result<CadenaHotelera, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<CadenaHotelera>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<CadenaHoteleraListItemDto>, i64), ApplicationError>;
}
