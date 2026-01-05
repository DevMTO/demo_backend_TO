use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::User;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserDetailDto {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserDetailDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            id_persona: user.id_persona,
            username: user.username,
            email: user.email,
            role: user.role.to_string(),
            id_entidad: user.id_entidad,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
        }
    }
}

/// Datos de persona para crear junto con el usuario (opcional)
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePersonaForUserRequest {
    #[validate(length(min = 1, max = 30))]
    pub tipo_documento: String,
    
    #[validate(length(min = 6, max = 20))]
    pub nro_documento: String,
    
    #[validate(length(min = 1, max = 100))]
    pub nombre: String,
    
    #[validate(length(min = 1, max = 100))]
    pub apellidos: String,
    
    #[validate(length(max = 20))]
    pub telefono: Option<String>,
    
    pub fecha_nacimiento: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreateUserRequest {
    /// ID de la persona ya registrada en el sistema (si se proporciona, no se crea persona nueva)
    pub id_persona: Option<i32>,
    
    /// Datos para crear una nueva persona (solo si id_persona es None)
    #[validate(nested)]
    pub nueva_persona: Option<CreatePersonaForUserRequest>,
    
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
    
    /// ID de la entidad asociada (agencia, transporte, etc.)
    pub id_entidad: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    
    pub role: Option<String>,
    
    pub is_active: Option<bool>,
    
    pub id_entidad: Option<i32>,
}

impl UpdateUserRequest {
    pub fn apply_to(self, mut user: crate::domain::entities::User, updated_by: Option<i32>) -> crate::domain::entities::User {
        use crate::domain::entities::UserRole;
        
        if let Some(email) = self.email {
            user.email = email.to_lowercase();
        }
        if let Some(role) = self.role {
            if let Ok(r) = role.parse::<UserRole>() {
                user.role = r;
            }
        }
        if let Some(is_active) = self.is_active {
            user.is_active = is_active;
        }
        if let Some(id_entidad) = self.id_entidad {
            user.id_entidad = Some(id_entidad);
        }
        user.updated_by = updated_by;
        user.updated_at = Utc::now();
        user
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserListItemDto {
    pub id: i32,
    pub nombre_completo: Option<String>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserListResponse {
    pub users: Vec<UserDetailDto>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
impl PaginationParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
    
    pub fn limit(&self) -> i64 {
        self.per_page.min(100) // Máximo 100 por página
    }
}

/// Request para que un superadmin cambie la contraseña de un usuario
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AdminChangePasswordRequest {
    #[validate(length(min = 8, message = "La contraseña debe tener al menos 8 caracteres"))]
    pub new_password: String,
}
