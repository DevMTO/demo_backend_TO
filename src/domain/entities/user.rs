//! # User Entity
//! 
//! Entidad de usuario del sistema. ID independiente del documento de identidad.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Roles del sistema con jerarquía de permisos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Administrador supremo con acceso total
    SuperAdmin,
    /// Administrador con acceso a gestión
    Admin,
    /// Sub-administrador con acceso limitado
    SubAdmin,
    /// Usuario regular
    User,
    /// Solo lectura
    Viewer,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::SubAdmin => write!(f, "subadmin"),
            UserRole::User => write!(f, "user"),
            UserRole::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "superadmin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "subadmin" => Ok(UserRole::SubAdmin),
            "user" => Ok(UserRole::User),
            "viewer" => Ok(UserRole::Viewer),
            _ => Err(format!("Invalid role: {s}")),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

/// Entidad de Usuario
/// 
/// Representa un usuario del sistema. El ID es un UUID, independiente
/// de cualquier documento de identidad por seguridad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// ID único del usuario (UUID)
    pub id: Uuid,
    /// Nombre de usuario único
    pub username: String,
    /// Email único
    pub email: String,
    /// Hash de la contraseña (nunca en texto plano)
    pub password_hash: String,
    /// Nombre para mostrar
    pub display_name: Option<String>,
    /// Rol del usuario en el sistema
    pub role: UserRole,
    /// Si el email ha sido verificado
    pub email_verified: bool,
    /// Si el usuario está activo
    pub is_active: bool,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de última actualización
    pub updated_at: DateTime<Utc>,
    /// Fecha del último login
    pub last_login: Option<DateTime<Utc>>,
    /// Usuario que creó este registro
    pub created_by: Option<String>,
    /// Usuario que actualizó este registro
    pub updated_by: Option<String>,
    /// Versión para control de concurrencia optimista
    pub version: i32,
    /// MFA habilitado
    pub mfa_enabled: bool,
    /// Secreto MFA (encriptado)
    pub mfa_secret: Option<String>,
    /// Códigos de respaldo MFA (encriptados)
    pub mfa_backup_codes: Option<serde_json::Value>,
}

impl User {
    /// Crear un nuevo usuario con valores por defecto
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            display_name: None,
            role,
            email_verified: false,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_login: None,
            created_by: None,
            updated_by: None,
            version: 1,
            mfa_enabled: false,
            mfa_secret: None,
            mfa_backup_codes: None,
        }
    }
    
    // ========================================
    // Métodos de verificación de roles
    // ========================================
    
    /// Verifica si es SuperAdmin
    pub fn is_superadmin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin)
    }
    
    /// Verifica si es Admin o superior
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si es SubAdmin o superior
    pub fn is_subadmin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::SubAdmin)
    }
    
    /// Verifica si puede gestionar usuarios
    pub fn can_manage_users(&self) -> bool {
        self.is_admin()
    }
    
    /// Verifica si puede acceder al panel de administración
    pub fn can_access_management(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::SubAdmin)
    }
    
    // ========================================
    // Métodos de mutación del estado
    // ========================================
    
    /// Actualiza el último login
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Activa el usuario
    pub fn activate(&mut self, updated_by: Option<String>) {
        self.is_active = true;
        self.updated_at = Utc::now();
        self.updated_by = updated_by;
    }
    
    /// Desactiva el usuario
    pub fn deactivate(&mut self, updated_by: Option<String>) {
        self.is_active = false;
        self.updated_at = Utc::now();
        self.updated_by = updated_by;
    }
    
    /// Verifica el email
    pub fn verify_email(&mut self) {
        self.email_verified = true;
        self.updated_at = Utc::now();
    }
    
    /// Actualiza el hash de la contraseña
    pub fn update_password(&mut self, new_password_hash: String, updated_by: Option<String>) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
        self.updated_by = updated_by;
        self.version += 1;
    }
    
    /// Habilita MFA
    pub fn enable_mfa(&mut self, secret: String, backup_codes: serde_json::Value) {
        self.mfa_enabled = true;
        self.mfa_secret = Some(secret);
        self.mfa_backup_codes = Some(backup_codes);
        self.updated_at = Utc::now();
    }
    
    /// Deshabilita MFA
    pub fn disable_mfa(&mut self) {
        self.mfa_enabled = false;
        self.mfa_secret = None;
        self.mfa_backup_codes = None;
        self.updated_at = Utc::now();
    }
}
