use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// UserStatus eliminado - ahora usamos is_active booleano

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Administrador supremo con acceso total al sistema (SYSCO)
    SuperAdmin,
    /// Administrador de agencia - gestiona files, tours, personal
    Admin,
    /// Personal de agencias - acceso a consultas y reportes
    Agencias,
    /// Empresa de transporte - gestiona vehículos y conductores
    Transportes,
    /// Conductor - ve sus asignaciones y puede aceptar/rechazar
    Conductores,
    /// Guía turístico - ve itinerarios asignados
    Guias,
    /// Restaurante - ve reservas de grupos
    Restaurantes,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Agencias => write!(f, "agencias"),
            UserRole::Transportes => write!(f, "transportes"),
            UserRole::Conductores => write!(f, "conductores"),
            UserRole::Guias => write!(f, "guias"),
            UserRole::Restaurantes => write!(f, "restaurantes"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "superadmin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "agencias" | "agencia" => Ok(UserRole::Agencias),
            "transportes" | "transporte" => Ok(UserRole::Transportes),
            "conductores" | "conductor" => Ok(UserRole::Conductores),
            "guias" | "guia" | "guide" => Ok(UserRole::Guias),
            "restaurantes" | "restaurante" | "restaurant" => Ok(UserRole::Restaurantes),
            _ => Err(format!("Invalid role: {s}")),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Agencias // Rol más común por defecto
    }
}

impl UserRole {
    /// Verifica si es SuperAdmin
    pub fn is_super_admin(&self) -> bool {
        matches!(self, UserRole::SuperAdmin)
    }
    
    /// Verifica si es Admin o superior
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si es Agencias o superior
    pub fn is_agencias(&self) -> bool {
        matches!(self, UserRole::SuperAdmin | UserRole::Admin | UserRole::Agencias)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// ID único del usuario (SERIAL en DB)
    pub id: i32,
    /// ID de la persona asociada (FK a persona)
    pub id_persona: Option<i32>,
    /// Nombre de usuario único
    pub username: String,
    /// Email único
    pub email: String,
    /// Hash de la contraseña (nunca en texto plano)
    pub password_hash: String,
    /// Rol del usuario en el sistema
    pub role: UserRole,
    /// ID de la entidad a la que pertenece (agencia, transporte, etc.)
    pub id_entidad: Option<i32>,
    /// Si el usuario está activo
    pub is_active: bool,
    /// Fecha del último login
    pub last_login: Option<DateTime<Utc>>,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de última actualización
    pub updated_at: DateTime<Utc>,
    /// ID del usuario que creó este registro
    pub created_by: Option<i32>,
    /// ID del usuario que actualizó este registro
    pub updated_by: Option<i32>,
}

impl User {
    /// Crear un nuevo usuario con valores por defecto (id será asignado por DB)
    pub fn new(
        id_persona: Option<i32>,
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Será asignado por la DB (SERIAL)
            id_persona,
            username,
            email,
            password_hash,
            role,
            id_entidad: None,
            is_active: true,
            last_login: None,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
    
    /// Crear usuario con entidad asociada
    pub fn with_entidad(
        id_persona: Option<i32>,
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
        id_entidad: i32,
    ) -> Self {
        let mut user = Self::new(id_persona, username, email, password_hash, role);
        user.id_entidad = Some(id_entidad);
        user
    }
    
    // ========================================
    // Métodos de verificación de roles
    // ========================================
    
    /// Verifica si es SuperAdmin (acceso total)
    pub fn is_superadmin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin)
    }
    
    /// Verifica si es Admin o superior
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si es personal de agencias (Admin, Agencias)
    pub fn is_agencia_staff(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Agencias)
    }
    
    /// Verifica si es empresa de transportes
    pub fn is_transporte(&self) -> bool {
        matches!(self.role, UserRole::Transportes)
    }
    
    /// Verifica si es conductor
    pub fn is_conductor(&self) -> bool {
        matches!(self.role, UserRole::Conductores)
    }
    
    /// Verifica si es guía
    pub fn is_guia(&self) -> bool {
        matches!(self.role, UserRole::Guias)
    }
    
    /// Verifica si es restaurante
    pub fn is_restaurante(&self) -> bool {
        matches!(self.role, UserRole::Restaurantes)
    }
    
    /// Verifica si puede gestionar usuarios
    pub fn can_manage_users(&self) -> bool {
        self.is_admin()
    }
    
    /// Verifica si puede gestionar files (tours activos)
    pub fn can_manage_files(&self) -> bool {
        self.is_admin()
    }
    
    /// Verifica si puede ver sus asignaciones (conductor/guía)
    pub fn can_view_assignments(&self) -> bool {
        matches!(self.role, UserRole::Conductores | UserRole::Guias | UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si puede aceptar/rechazar asignaciones
    pub fn can_respond_assignments(&self) -> bool {
        matches!(self.role, UserRole::Conductores | UserRole::Guias)
    }
    
    /// Verifica si puede acceder al panel de administración
    pub fn can_access_management(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Agencias)
    }
    
    /// Verifica si puede gestionar vehículos
    pub fn can_manage_vehicles(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Transportes)
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
        self.is_active = true;
        self.updated_at = Utc::now();
    }
    
    /// Desactiva el usuario
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
    
    /// Actualiza el hash de la contraseña
    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }
    
    /// Actualiza la entidad asociada
    pub fn set_entidad(&mut self, id_entidad: i32) {
        self.id_entidad = Some(id_entidad);
        self.updated_at = Utc::now();
    }
    
    /// Remueve la entidad asociada
    pub fn clear_entidad(&mut self) {
        self.id_entidad = None;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<i32>,
    pub is_active: bool,
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
            is_active: user.is_active,
        }
    }
}
