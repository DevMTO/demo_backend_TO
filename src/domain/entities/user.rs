//! # User Entity
//! 
//! Entidad de usuario del sistema según el diagrama de base de datos.
//! Usuario vinculado a una persona y una entidad (agencia, transporte, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status del usuario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserStatus {
    Activo,
    Inactivo,
    Suspendido,
    PendienteVerificacion,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Activo => write!(f, "activo"),
            UserStatus::Inactivo => write!(f, "inactivo"),
            UserStatus::Suspendido => write!(f, "suspendido"),
            UserStatus::PendienteVerificacion => write!(f, "pendiente_verificacion"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "activo" => Ok(UserStatus::Activo),
            "inactivo" => Ok(UserStatus::Inactivo),
            "suspendido" => Ok(UserStatus::Suspendido),
            "pendiente_verificacion" => Ok(UserStatus::PendienteVerificacion),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::PendienteVerificacion
    }
}

/// Roles del sistema con jerarquía de permisos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Administrador supremo con acceso total
    SuperAdmin,
    /// Administrador de agencia
    Admin,
    /// Sub-administrador con acceso limitado
    SubAdmin,
    /// Operador/Usuario regular
    Operador,
    /// Solo lectura
    Viewer,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::SubAdmin => write!(f, "subadmin"),
            UserRole::Operador => write!(f, "operador"),
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
            "operador" | "user" => Ok(UserRole::Operador),
            "viewer" => Ok(UserRole::Viewer),
            _ => Err(format!("Invalid role: {s}")),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Operador
    }
}

/// Entidad de Usuario según diagrama
/// 
/// Campos: id, id_persona, username, email, password, role, id_entidad, nombre_entidad, status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// ID único del usuario (integer en DB, UUID internamente)
    pub id: Uuid,
    /// ID de la persona asociada (FK a persona)
    pub id_persona: Uuid,
    /// Nombre de usuario único
    pub username: String,
    /// Email único
    pub email: String,
    /// Hash de la contraseña (nunca en texto plano)
    pub password_hash: String,
    /// Rol del usuario en el sistema
    pub role: UserRole,
    /// ID de la entidad a la que pertenece (agencia, transporte, etc.)
    pub id_entidad: Option<Uuid>,
    /// Nombre de la entidad (tipo: agencia, transporte, etc.)
    pub nombre_entidad: Option<String>,
    /// Status del usuario
    pub status: UserStatus,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de última actualización
    pub updated_at: DateTime<Utc>,
    /// Fecha del último login
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    /// Crear un nuevo usuario con valores por defecto
    pub fn new(
        id_persona: Uuid,
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            id_persona,
            username,
            email,
            password_hash,
            role,
            id_entidad: None,
            nombre_entidad: None,
            status: UserStatus::PendienteVerificacion,
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }
    
    /// Crear usuario con entidad asociada
    pub fn with_entidad(
        id_persona: Uuid,
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
        id_entidad: Uuid,
        nombre_entidad: String,
    ) -> Self {
        let mut user = Self::new(id_persona, username, email, password_hash, role);
        user.id_entidad = Some(id_entidad);
        user.nombre_entidad = Some(nombre_entidad);
        user
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
    
    /// Verifica si el usuario está activo
    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Activo)
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
    pub fn activate(&mut self) {
        self.status = UserStatus::Activo;
        self.updated_at = Utc::now();
    }
    
    /// Desactiva el usuario
    pub fn deactivate(&mut self) {
        self.status = UserStatus::Inactivo;
        self.updated_at = Utc::now();
    }
    
    /// Suspende el usuario
    pub fn suspend(&mut self) {
        self.status = UserStatus::Suspendido;
        self.updated_at = Utc::now();
    }
    
    /// Actualiza el hash de la contraseña
    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }
    
    /// Actualiza la entidad asociada
    pub fn set_entidad(&mut self, id_entidad: Uuid, nombre_entidad: String) {
        self.id_entidad = Some(id_entidad);
        self.nombre_entidad = Some(nombre_entidad);
        self.updated_at = Utc::now();
    }
    
    /// Remueve la entidad asociada
    pub fn clear_entidad(&mut self) {
        self.id_entidad = None;
        self.nombre_entidad = None;
        self.updated_at = Utc::now();
    }
}

/// Información del usuario para respuestas API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub id_persona: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<Uuid>,
    pub nombre_entidad: Option<String>,
    pub status: String,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            id_persona: user.id_persona,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.to_string(),
            id_entidad: user.id_entidad,
            nombre_entidad: user.nombre_entidad.clone(),
            status: user.status.to_string(),
        }
    }
}
