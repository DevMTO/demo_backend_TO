# 🎓 Guía Completa de Arquitectura Hexagonal para Principiantes

## TourOperator - Sistema de Gestión de Pasajeros

> Una guía paso a paso para entender la arquitectura hexagonal, explicada desde cero.

---

## 📚 Tabla de Contenidos

1. [¿Qué es la Arquitectura Hexagonal?](#qué-es-la-arquitectura-hexagonal)
2. [Comparación con Arquitectura Tradicional](#comparación-con-arquitectura-tradicional)
3. [Las 4 Capas Explicadas](#las-4-capas-explicadas)
4. [Ejemplo Práctico Completo](#ejemplo-práctico-completo)
5. [Flujo de una Petición HTTP](#flujo-de-una-petición-http)
6. [Glosario de Términos](#glosario-de-términos)

---

## 🤔 ¿Qué es la Arquitectura Hexagonal?

### La Analogía del Restaurante 🍽️

Imagina un **restaurante**:

| Parte del Restaurante | Equivalente en Código |
|----------------------|----------------------|
| 👨‍🍳 **Cocina** (donde se prepara la comida) | **Domain** (lógica de negocio) |
| 📝 **Recetas** (instrucciones de preparación) | **Application** (casos de uso) |
| 🚪 **Entrada** (meseros que reciben pedidos) | **Presentation** (API/HTTP) |
| 🏪 **Proveedores** (quienes traen ingredientes) | **Infrastructure** (BD, servicios externos) |

**El punto clave**: La cocina (Domain) NO sabe de dónde vienen los ingredientes ni quién los pide. Solo sabe cocinar.

### Diagrama Simple

```
┌─────────────────────────────────────────────────────────────┐
│                    TU APLICACIÓN                             │
│                                                              │
│    ┌─────────────────────────────────────────────────┐      │
│    │              PRESENTATION                        │      │
│    │   (Recibe peticiones HTTP, valida, responde)    │      │
│    └──────────────────────┬──────────────────────────┘      │
│                           │                                  │
│                           ▼                                  │
│    ┌─────────────────────────────────────────────────┐      │
│    │              APPLICATION                         │      │
│    │   (Orquesta la lógica, coordina operaciones)    │      │
│    └──────────────────────┬──────────────────────────┘      │
│                           │                                  │
│                           ▼                                  │
│    ┌─────────────────────────────────────────────────┐      │
│    │                 DOMAIN                           │      │
│    │   (Reglas de negocio puras, sin dependencias)   │      │
│    │   ⭐ EL CORAZÓN DE TU APLICACIÓN ⭐            │      │
│    └──────────────────────┬──────────────────────────┘      │
│                           │                                  │
│                           ▼                                  │
│    ┌─────────────────────────────────────────────────┐      │
│    │             INFRASTRUCTURE                       │      │
│    │   (Base de datos, emails, servicios externos)   │      │
│    └─────────────────────────────────────────────────┘      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## ⚖️ Comparación con Arquitectura Tradicional

### ❌ Arquitectura Tradicional (MVC Típico)

```
📁 proyecto/
├── 📂 controllers/
│   └── UserController.rs      # Maneja HTTP Y lógica de negocio
├── 📂 models/
│   └── User.rs                # Modelo con SQL directo
└── 📂 views/
    └── user.html              # Vistas
```

**Problema**: Todo está mezclado. El controlador conoce la base de datos directamente.

```rust
// ❌ Arquitectura Tradicional - TODO JUNTO
fn create_user(request: Request) -> Response {
    // 1. Validar request (OK)
    let username = request.get("username");
    
    // 2. Lógica de negocio mezclada con SQL (MAL)
    let sql = "INSERT INTO users (username) VALUES (?)";
    database.execute(sql, [username]);  // 👎 SQL directo en controlador
    
    // 3. Retornar respuesta
    Response::ok()
}
```

### ✅ Arquitectura Hexagonal

```
📁 proyecto/
├── 📂 presentation/      # Solo HTTP
│   └── handlers/
├── 📂 application/       # Casos de uso
│   └── services/
├── 📂 domain/            # Lógica pura
│   ├── entities/
│   └── traits/           # Interfaces/Contratos
└── 📂 infrastructure/    # Implementaciones
    └── persistence/
```

```rust
// ✅ Arquitectura Hexagonal - SEPARADO

// presentation/handlers/user_handler.rs
fn create_user(request: Request, service: UserService) -> Response {
    let dto = request.parse::<CreateUserRequest>()?;
    let user = service.create_user(dto)?;  // Delega al servicio
    Response::ok(user)
}

// application/services/user_service.rs
impl UserService {
    fn create_user(&self, dto: CreateUserRequest) -> Result<User> {
        let user = User::new(dto.username);  // Usa entidad de dominio
        self.repository.save(user)           // Delega al repositorio
    }
}

// domain/entities/user.rs
struct User {
    id: Uuid,
    username: String,
}

// infrastructure/persistence/postgres_user_repo.rs
impl UserRepository for PostgresUserRepo {
    fn save(&self, user: User) -> Result<User> {
        // Aquí SÍ está el SQL, pero aislado
        diesel::insert_into(users::table)
            .values(&user)
            .execute(&self.conn)?;
        Ok(user)
    }
}
```

---

## 🏗️ Las 4 Capas Explicadas

### 1. 🔴 PRESENTATION (Capa de Presentación)

**¿Qué hace?** Recibe peticiones HTTP y devuelve respuestas.

**Analogía**: Es el **mesero** del restaurante. Recibe el pedido, lo pasa a la cocina, y entrega el plato.

**Archivos típicos**:
```
src/presentation/
├── routes.rs          # Define las URLs
├── handlers/          # Maneja cada endpoint
│   ├── auth_handler.rs
│   └── user_handler.rs
└── middleware/        # Autenticación, logging
    └── auth_middleware.rs
```

**Ejemplo real**:

```rust
// src/presentation/handlers/auth_handler.rs

/// POST /api/v1/auth/login
/// 
/// Esta función SOLO hace 3 cosas:
/// 1. Recibe el JSON del request
/// 2. Llama al servicio de aplicación
/// 3. Devuelve la respuesta HTTP
pub async fn login(
    State(container): State<DependencyContainer>,  // Inyección de dependencias
    Json(request): Json<LoginRequest>,              // Parsea el body JSON
) -> Result<Json<AuthResponse>, AppError> {
    
    // Validar datos de entrada
    request.validate()?;
    
    // ⬇️ Delega TODO el trabajo al servicio de aplicación
    let response = container
        .auth_service
        .login(request.identifier, request.password)
        .await?;
    
    // Retornar respuesta HTTP
    Ok(Json(response))
}
```

**Reglas de esta capa**:
- ✅ Parsear JSON de requests
- ✅ Validar datos de entrada
- ✅ Llamar a servicios de aplicación
- ✅ Formatear respuestas HTTP
- ❌ NO hacer lógica de negocio
- ❌ NO acceder a la base de datos directamente

---

### 2. 🟢 APPLICATION (Capa de Aplicación)

**¿Qué hace?** Orquesta la lógica de negocio. Coordina las operaciones.

**Analogía**: Es el **jefe de cocina** que coordina a todos los cocineros.

**Archivos típicos**:
```
src/application/
├── services/          # Casos de uso
│   ├── auth_service.rs
│   └── user_service.rs
└── dtos/              # Objetos de transferencia
    ├── auth_dto.rs
    └── user_dto.rs
```

**Ejemplo real**:

```rust
// src/application/services/auth_service.rs

/// Servicio de Autenticación
/// 
/// Este servicio COORDINA las operaciones, pero NO implementa
/// los detalles de base de datos ni seguridad.
pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,      // Interfaz, no implementación
    session_repository: Arc<dyn SessionRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
}

impl AuthService {
    /// Caso de uso: Login de usuario
    pub async fn login(
        &self,
        identifier: String,
        password: String,
    ) -> Result<AuthResponse, AppError> {
        
        // 1. Buscar usuario (delega al repositorio)
        let user = self.user_repository
            .find_by_email_or_username(&identifier)
            .await?
            .ok_or(AppError::InvalidCredentials)?;
        
        // 2. Verificar contraseña (delega al hasher)
        if !self.password_hasher.verify(&password, &user.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }
        
        // 3. Verificar que usuario esté activo (regla de negocio)
        if !user.is_active {
            return Err(AppError::UserDisabled);
        }
        
        // 4. Crear sesión (delega al repositorio)
        let session = self.session_repository
            .create(user.id, Duration::hours(24))
            .await?;
        
        // 5. Retornar respuesta
        Ok(AuthResponse {
            user: UserInfo::from(user),
            session_id: session.id,
            expires_in: 86400,
        })
    }
}
```

**Reglas de esta capa**:
- ✅ Coordinar operaciones
- ✅ Aplicar reglas de negocio simples
- ✅ Llamar a repositorios (a través de interfaces)
- ✅ Transformar entidades a DTOs
- ❌ NO conocer detalles de HTTP
- ❌ NO conocer SQL ni detalles de BD

---

### 3. 🟡 DOMAIN (Capa de Dominio)

**¿Qué hace?** Contiene la lógica de negocio PURA. Las reglas del negocio.

**Analogía**: Son las **recetas** del restaurante. Las reglas de cómo se prepara cada plato.

**Archivos típicos**:
```
src/domain/
├── entities/          # Objetos de negocio
│   ├── user.rs
│   └── session.rs
├── errors/            # Errores de dominio
│   └── mod.rs
└── traits/            # Interfaces/Contratos (PORTS)
    ├── user_repository.rs
    └── password_hasher.rs
```

**Ejemplo real - Entidad**:

```rust
// src/domain/entities/user.rs

/// Entidad de Usuario
/// 
/// Esta es una entidad de DOMINIO. No sabe nada de:
/// - HTTP
/// - Base de datos
/// - JSON
/// 
/// Solo contiene datos y reglas de negocio.
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Roles del sistema
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    SuperAdmin,  // Puede hacer TODO
    Admin,       // Puede gestionar usuarios
    SubAdmin,    // Acceso limitado
    User,        // Usuario normal
    Viewer,      // Solo lectura
}

impl User {
    /// Constructor con validaciones
    pub fn new(username: String, email: String, password_hash: String) -> Result<Self, DomainError> {
        // Regla de negocio: username mínimo 3 caracteres
        if username.len() < 3 {
            return Err(DomainError::InvalidUsername);
        }
        
        Ok(Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role: UserRole::User,  // Por defecto es User
            is_active: true,
            created_at: Utc::now(),
        })
    }
    
    /// Regla de negocio: ¿Puede este usuario gestionar otros usuarios?
    pub fn can_manage_users(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin)
    }
    
    /// Regla de negocio: ¿Puede este usuario ver reportes financieros?
    pub fn can_view_financial_reports(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::SubAdmin)
    }
}
```

**Ejemplo real - Trait (Interfaz/Puerto)**:

```rust
// src/domain/traits/user_repository.rs

/// Este TRAIT define el CONTRATO.
/// Es como un "contrato" que dice QUÉ operaciones necesitamos,
/// pero NO dice CÓMO se implementan.
/// 
/// Esto es un "PORT" en arquitectura hexagonal.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Buscar usuario por ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    
    /// Buscar usuario por email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    
    /// Guardar un nuevo usuario
    async fn save(&self, user: User) -> Result<User, DomainError>;
    
    /// Actualizar usuario existente
    async fn update(&self, user: User) -> Result<User, DomainError>;
}

/// Trait para hashear contraseñas
pub trait PasswordHasher: Send + Sync {
    /// Hashear una contraseña
    fn hash(&self, password: &str) -> Result<String, DomainError>;
    
    /// Verificar si una contraseña coincide con su hash
    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError>;
}
```

**Reglas de esta capa**:
- ✅ Definir entidades de negocio
- ✅ Definir reglas de negocio
- ✅ Definir interfaces (traits/ports)
- ❌ NO conocer frameworks
- ❌ NO conocer base de datos
- ❌ NO tener dependencias externas

---

### 4. 🔵 INFRASTRUCTURE (Capa de Infraestructura)

**¿Qué hace?** Implementa las interfaces del dominio con tecnologías específicas.

**Analogía**: Son los **proveedores** del restaurante. Implementan cómo conseguir los ingredientes.

**Archivos típicos**:
```
src/infrastructure/
├── persistence/           # Implementaciones de BD
│   ├── database.rs        # Conexión a PostgreSQL
│   ├── postgres_user_repo.rs
│   └── postgres_session_repo.rs
├── security/              # Implementaciones de seguridad
│   ├── argon2_hasher.rs   # Implementación de PasswordHasher
│   └── session_token.rs
└── container.rs           # Inyección de dependencias
```

**Ejemplo real - Implementación de Repositorio**:

```rust
// src/infrastructure/persistence/postgres_user_repo.rs

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

/// Implementación de UserRepository usando PostgreSQL + Diesel
/// 
/// Esta estructura IMPLEMENTA el trait (interfaz) del dominio.
/// Aquí SÍ conocemos los detalles de la base de datos.
pub struct PostgresUserRepository {
    pool: Pool<AsyncPgConnection>,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }
}

/// Implementamos el trait del dominio
#[async_trait]
impl UserRepository for PostgresUserRepository {
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.get().await?;
        
        // Aquí SÍ usamos SQL (a través de Diesel)
        let result = users::table
            .filter(users::id.eq(id))
            .first::<UserModel>(&mut conn)
            .await
            .optional()?;
        
        // Convertimos el modelo de DB a entidad de dominio
        Ok(result.map(|m| m.into()))
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.get().await?;
        
        let result = users::table
            .filter(users::email.eq(email))
            .first::<UserModel>(&mut conn)
            .await
            .optional()?;
        
        Ok(result.map(|m| m.into()))
    }
    
    async fn save(&self, user: User) -> Result<User, DomainError> {
        let mut conn = self.pool.get().await?;
        
        // Convertimos entidad de dominio a modelo de DB
        let model = UserModel::from(user);
        
        diesel::insert_into(users::table)
            .values(&model)
            .returning(UserModel::as_returning())
            .get_result(&mut conn)
            .await
            .map(|m| m.into())
            .map_err(|e| DomainError::Database(e.to_string()))
    }
}
```

**Ejemplo real - Implementación de PasswordHasher**:

```rust
// src/infrastructure/security/argon2_hasher.rs

use argon2::{Argon2, PasswordHasher as ArgonHasher, PasswordVerifier};

/// Implementación de PasswordHasher usando Argon2id
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        // Configuración segura de Argon2id
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(65536, 3, 4, None).unwrap(),
        );
        Self { argon2 }
    }
}

/// Implementamos el trait del dominio
impl PasswordHasher for Argon2PasswordHasher {
    
    fn hash(&self, password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| DomainError::Security(e.to_string()))
    }
    
    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DomainError::Security(e.to_string()))?;
        
        Ok(self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
```

**Reglas de esta capa**:
- ✅ Implementar interfaces del dominio
- ✅ Conocer detalles de tecnologías (SQL, HTTP clients, etc.)
- ✅ Manejar conexiones y recursos externos
- ❌ NO definir reglas de negocio

---

## 🔄 Flujo de una Petición HTTP

### Diagrama Paso a Paso

```
   CLIENTE (Frontend/Postman)
         │
         │  POST /api/v1/auth/login
         │  {"identifier": "admin", "password": "123456"}
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  1️⃣ PRESENTATION LAYER                                      │
│     routes.rs → auth_handler.rs                              │
│                                                              │
│     • Parsea JSON del body                                   │
│     • Valida que los campos existan                          │
│     • Llama al AuthService                                   │
└─────────────────────────────────────────────────────────────┘
         │
         │  LoginRequest { identifier: "admin", password: "123456" }
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  2️⃣ APPLICATION LAYER                                       │
│     auth_service.rs                                          │
│                                                              │
│     • Busca usuario por email/username                       │
│     • Verifica contraseña                                    │
│     • Verifica que usuario esté activo                       │
│     • Crea sesión                                            │
│     • Retorna AuthResponse                                   │
└─────────────────────────────────────────────────────────────┘
         │
         │  Llama a: UserRepository, PasswordHasher, SessionRepository
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  3️⃣ DOMAIN LAYER                                            │
│     User entity, UserRole enum                               │
│                                                              │
│     • Valida reglas de negocio                               │
│     • user.is_active debe ser true                           │
│     • user.role determina permisos                           │
└─────────────────────────────────────────────────────────────┘
         │
         │  Traits: UserRepository, PasswordHasher
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  4️⃣ INFRASTRUCTURE LAYER                                    │
│     postgres_user_repo.rs, argon2_hasher.rs                  │
│                                                              │
│     • Ejecuta SQL en PostgreSQL                              │
│     • Verifica hash con Argon2id                             │
│     • Guarda sesión en DB                                    │
└─────────────────────────────────────────────────────────────┘
         │
         │  Respuesta sube por las capas
         │
         ▼
   CLIENTE
         │
         │  200 OK
         │  {"user": {...}, "session_id": "...", "expires_in": 86400}
         │  Cookie: session_token=abc123...
```

---

## 📖 Glosario de Términos

| Término | Significado |
|---------|-------------|
| **Entity** | Objeto de negocio con identidad (User, Session, Viaje) |
| **DTO** | Data Transfer Object - objeto para transferir datos entre capas |
| **Repository** | Interfaz para acceso a datos |
| **Service** | Clase que orquesta casos de uso |
| **Handler** | Función que maneja una petición HTTP |
| **Trait** | Interfaz en Rust (contrato sin implementación) |
| **Port** | Interfaz que define cómo el dominio se comunica con el exterior |
| **Adapter** | Implementación concreta de un Port |
| **Use Case** | Una operación de negocio (login, crear usuario, etc.) |
| **Dependency Injection** | Patrón para pasar dependencias desde afuera |

---

## ❓ Preguntas Frecuentes

### ¿Por qué tantas capas? ¿No es más código?

Sí, es más código inicial. Pero:

1. **Facilita testing**: Puedes probar la lógica de negocio sin base de datos
2. **Facilita cambios**: Puedes cambiar PostgreSQL por MySQL sin tocar la lógica
3. **Código más limpio**: Cada archivo tiene una responsabilidad clara
4. **Trabajo en equipo**: Diferentes personas pueden trabajar en diferentes capas

### ¿Cuándo NO usar arquitectura hexagonal?

- Proyectos muy pequeños (scripts, prototipos rápidos)
- Proyectos con deadline muy corto
- Cuando solo una persona trabajará en el código

### ¿Cómo sé en qué capa poner mi código?

Pregúntate:

1. **¿Maneja HTTP?** → Presentation
2. **¿Coordina operaciones?** → Application
3. **¿Es una regla de negocio?** → Domain
4. **¿Accede a recursos externos?** → Infrastructure

---

*Documentación creada para principiantes - TourOperator*
*Diciembre 2024*
