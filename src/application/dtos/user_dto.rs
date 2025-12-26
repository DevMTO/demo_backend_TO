//! # User DTOs
//! 
//! Data Transfer Objects para usuarios.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::entities::User;

/// Información completa del usuario (para administradores)
#[derive(Debug, Clone, Serialize)]
pub struct UserDetailDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub mfa_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserDetailDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role.to_string(),
            email_verified: user.email_verified,
            is_active: user.is_active,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
        }
    }
}

/// Request para crear usuario
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    pub display_name: Option<String>,
    
    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
}

/// Request para actualizar usuario
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    
    pub role: Option<String>,
    
    pub is_active: Option<bool>,
}

/// Lista paginada de usuarios
#[derive(Debug, Clone, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserDetailDto>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl UserListResponse {
    pub fn new(users: Vec<UserDetailDto>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            users,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Parámetros de paginación
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
    
    pub fn limit(&self) -> i64 {
        self.per_page.min(100) // Máximo 100 por página
    }
}
