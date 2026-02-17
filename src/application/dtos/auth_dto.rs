use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Identifier is required"))]
    pub identifier: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    
    #[serde(default)]
    pub remember_me: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct RegisterRequest {
    pub id_persona: Option<i32>,
    
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub password_confirm: String,
    
    pub role: Option<String>,
    
    pub id_entidad: Option<i32>,
    
    pub nombre_entidad: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AuthResponse {
    pub user: AuthUserInfo,
    pub session_id: i32,
    pub expires_in: i64,
    pub extended_session: bool,
}

impl AuthResponse {
    pub fn new(user: AuthUserInfo, session_id: i32, expires_in: i64, extended_session: bool) -> Self {
        Self {
            user,
            session_id,
            expires_in,
            extended_session,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct AuthUserInfo {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
    
    #[validate(must_match(other = "new_password", message = "Passwords do not match"))]
    pub new_password_confirm: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct LogoutRequest {
    #[serde(default)]
    pub all_sessions: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}



/// Información de persona asociada al usuario (para perfil)
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PersonaProfileInfo {
    pub id: i32,
    pub tipo_documento: String,
    pub nro_documento: String,
    pub nombre: String,
    pub apellidos: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
}

/// Respuesta completa del perfil de usuario con su persona asociada
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UserProfileResponse {
    pub user: AuthUserInfo,
    pub persona: Option<PersonaProfileInfo>,
}

/// Request para actualizar el perfil del usuario
#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateProfileRequest {
    /// Nombre (de la persona)
    #[validate(length(min = 2, max = 100, message = "Nombre debe tener entre 2 y 100 caracteres"))]
    pub nombre: Option<String>,
    
    /// Apellidos (de la persona)
    #[validate(length(min = 2, max = 100, message = "Apellidos debe tener entre 2 y 100 caracteres"))]
    pub apellidos: Option<String>,
    
    /// Teléfono (de la persona)
    #[validate(length(max = 20, message = "Teléfono muy largo"))]
    pub telefono: Option<String>,
    
    /// Correo electrónico personal (de la persona, diferente al email de login)
    #[validate(email(message = "Correo inválido"))]
    pub correo: Option<String>,
    
    /// Fecha de nacimiento
    pub fecha_nacimiento: Option<NaiveDate>,
}
