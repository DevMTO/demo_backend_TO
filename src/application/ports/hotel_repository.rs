use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Hotel;
use crate::application::dtos::HotelListItemDto;

#[async_trait]
pub trait HotelRepositoryPort: Send + Sync {
    async fn create(&self, hotel: &Hotel) -> Result<Hotel, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Hotel>, ApplicationError>;
    async fn update(&self, hotel: &Hotel) -> Result<Hotel, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Hotel>, ApplicationError>;
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn list_by_cadena(&self, id_cadena: i32, limit: i64, offset: i64) -> Result<Vec<Hotel>, ApplicationError>;
    async fn count_by_cadena(&self, id_cadena: i32) -> Result<i64, ApplicationError>;
    async fn list_by_cadena_with_details(&self, id_cadena: i32, limit: i64, offset: i64) -> Result<(Vec<HotelListItemDto>, i64), ApplicationError>;
    async fn list_with_cadena(&self, limit: i64, offset: i64) -> Result<(Vec<HotelListItemDto>, i64), ApplicationError>;
}
