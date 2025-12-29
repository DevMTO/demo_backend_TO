use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Administrador supremo con acceso total al sistema (SYSCO)
    SuperAdmin,
    /// Administrador de agencia - gestiona files, tours, personal
    Admin,
    /// Personal de agencia - acceso a consultas y reportes
    Agencia,
    /// Empresa de transporte - gestiona vehículos y conductores
    Transportes,
    /// Conductor - ve sus asignaciones y puede aceptar/rechazar
    Conductor,
    /// Guía turístico - ve itinerarios asignados
    Guia,
    /// Restaurante - ve reservas de grupos
    Restaurante,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Agencia => write!(f, "agencia"),
            UserRole::Transportes => write!(f, "transportes"),
            UserRole::Conductor => write!(f, "conductor"),
            UserRole::Guia => write!(f, "guia"),
            UserRole::Restaurante => write!(f, "restaurante"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "superadmin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "agencia" => Ok(UserRole::Agencia),
            "transportes" | "transporte" => Ok(UserRole::Transportes),
            "conductor" => Ok(UserRole::Conductor),
            "guia" | "guide" => Ok(UserRole::Guia),
            "restaurante" | "restaurant" => Ok(UserRole::Restaurante),
            _ => Err(format!("Invalid role: {s}")),
        }
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Agencia // Rol más común por defecto
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
    /// Nombre de la entidad (tipo: agencia, transporte, etc.)
    pub nombre_entidad: Option<String>,
    /// Status del usuario
    pub status: UserStatus,
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
            nombre_entidad: None,
            status: UserStatus::PendienteVerificacion,
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
    
    /// Verifica si es SuperAdmin (acceso total)
    pub fn is_superadmin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin)
    }
    
    /// Verifica si es Admin o superior
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si es personal de agencia (Admin, Agencia)
    pub fn is_agencia_staff(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Agencia)
    }
    
    /// Verifica si es empresa de transportes
    pub fn is_transporte(&self) -> bool {
        matches!(self.role, UserRole::Transportes)
    }
    
    /// Verifica si es conductor
    pub fn is_conductor(&self) -> bool {
        matches!(self.role, UserRole::Conductor)
    }
    
    /// Verifica si es guía
    pub fn is_guia(&self) -> bool {
        matches!(self.role, UserRole::Guia)
    }
    
    /// Verifica si es restaurante
    pub fn is_restaurante(&self) -> bool {
        matches!(self.role, UserRole::Restaurante)
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
        matches!(self.role, UserRole::Conductor | UserRole::Guia | UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Verifica si puede aceptar/rechazar asignaciones
    pub fn can_respond_assignments(&self) -> bool {
        matches!(self.role, UserRole::Conductor | UserRole::Guia)
    }
    
    /// Verifica si puede acceder al panel de administración
    pub fn can_access_management(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Agencia)
    }
    
    /// Verifica si puede gestionar vehículos
    pub fn can_manage_vehicles(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Transportes)
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
    pub fn set_entidad(&mut self, id_entidad: i32, nombre_entidad: String) {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub id_persona: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub id_entidad: Option<i32>,
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
