# 🚌 TourOperator - Backend Architecture Documentation

## Sistema de Gestión de Pasajeros para Transporte Turístico

> Documentación de arquitectura hexagonal del backend en Rust

---

## 📋 Tabla de Contenidos

1. [Visión General](#visión-general)
2. [Arquitectura Hexagonal](#arquitectura-hexagonal)
3. [Estructura de Directorios](#estructura-de-directorios)
4. [Capas de la Arquitectura](#capas-de-la-arquitectura)
5. [Flujo de Datos](#flujo-de-datos)
6. [Autenticación y Seguridad](#autenticación-y-seguridad)
7. [Generación de Tipos TypeScript](#generación-de-tipos-typescript)
8. [API Endpoints](#api-endpoints)
9. [Guía de Desarrollo](#guía-de-desarrollo)

---

## 🎯 Visión General

**TourOperator** es un sistema de gestión de pasajeros para empresas de transporte turístico (similar a Cruz del Sur). El backend está construido en Rust siguiendo los principios de **Arquitectura Hexagonal** (Ports & Adapters).

### Stack Tecnológico

| Componente | Tecnología | Versión |
|------------|------------|---------|
| Lenguaje | Rust | 2021 Edition |
| Framework Web | Axum | 0.8 |
| Base de Datos | PostgreSQL | 14+ |
| ORM | Diesel (async) | 2.2 |
| Pool de Conexiones | Deadpool | 0.12 |
| Autenticación | Session Cookies | - |
| Password Hashing | Argon2id | 0.5 |
| Serialización | Serde | 1.0 |
| Type Export | ts-rs | 10.0 |

---

## 🔷 Arquitectura Hexagonal

```
┌─────────────────────────────────────────────────────────────────┐
│                        PRESENTATION                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Routes    │  │  Handlers   │  │  Middleware (Auth, CORS)│  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
└─────────┼────────────────┼─────────────────────┼────────────────┘
          │                │                     │
          ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        APPLICATION                               │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │   Services          │  │        DTOs (Data Transfer)     │   │
│  │   (Use Cases)       │  │   ┌─────────────────────────┐   │   │
│  │                     │  │   │   ts_types.rs → TS      │   │   │
│  │   - AuthService     │  │   │   (auto-generated)      │   │   │
│  │   - UserService     │  │   └─────────────────────────┘   │   │
│  │   - ViajeService    │  │                                 │   │
│  └──────────┬──────────┘  └─────────────────────────────────┘   │
└─────────────┼───────────────────────────────────────────────────┘
              │
              ▼ (Ports - Interfaces/Traits)
┌─────────────────────────────────────────────────────────────────┐
│                          DOMAIN                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Entities   │  │   Errors    │  │     Value Objects       │  │
│  │             │  │             │  │                         │  │
│  │  - User     │  │  - AppError │  │  - Email               │  │
│  │  - Session  │  │  - AuthErr  │  │  - Password            │  │
│  │  - Viaje    │  │  - DomainErr│  │  - DocumentNumber      │  │
│  │  - Pasajero │  │             │  │                         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                 PORTS (Traits/Interfaces)                    ││
│  │                                                              ││
│  │  trait UserRepository { ... }                                ││
│  │  trait SessionRepository { ... }                             ││
│  │  trait PasswordHasher { ... }                                ││
│  │  trait TokenProvider { ... }                                 ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
              ▲
              │ (Adapters - Implementations)
┌─────────────────────────────────────────────────────────────────┐
│                      INFRASTRUCTURE                              │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │   Persistence       │  │       Security                   │   │
│  │                     │  │                                  │   │
│  │   - PostgresUser    │  │   - Argon2PasswordHasher        │   │
│  │   - PostgresSession │  │   - SessionTokenProvider        │   │
│  │   - PostgresViaje   │  │                                  │   │
│  └─────────────────────┘  └──────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │   External Services │  │       Database                   │   │
│  │                     │  │                                  │   │
│  │   - TigrisClient    │  │   - Diesel Async                │   │
│  │   - (S3 Storage)    │  │   - Deadpool                    │   │
│  └─────────────────────┘  └──────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📁 Estructura de Directorios

```
backend/
├── src/
│   ├── main.rs                     # Entry point
│   ├── config.rs                   # Configuration
│   │
│   ├── domain/                     # 🟡 CORE - Business Logic
│   │   ├── mod.rs
│   │   ├── entities/               # Business entities
│   │   │   ├── user.rs             # User entity + UserRole
│   │   │   ├── session.rs          # Session entity
│   │   │   └── mod.rs
│   │   ├── errors/                 # Domain errors
│   │   │   ├── application_error.rs
│   │   │   └── mod.rs
│   │   ├── services/               # Domain services (ports)
│   │   │   └── mod.rs
│   │   └── value_objects/          # Value objects
│   │       └── mod.rs
│   │
│   ├── application/                # 🟢 USE CASES
│   │   ├── mod.rs
│   │   ├── dtos/                   # Data Transfer Objects
│   │   │   ├── auth_dto.rs         # Auth DTOs
│   │   │   ├── user_dto.rs         # User DTOs
│   │   │   ├── ts_types.rs         # 🚀 TS-RS exports
│   │   │   └── mod.rs
│   │   └── services/               # Application services
│   │       ├── auth_service.rs     # Auth use cases
│   │       ├── user_service.rs     # User management
│   │       └── mod.rs
│   │
│   ├── infrastructure/             # 🔵 ADAPTERS (External)
│   │   ├── mod.rs
│   │   ├── container.rs            # Dependency injection
│   │   ├── persistence/            # Database adapters
│   │   │   ├── database.rs         # Pool & connection
│   │   │   ├── user_repository.rs  # PostgresUserRepo
│   │   │   ├── session_repository.rs
│   │   │   └── mod.rs
│   │   └── security/               # Security adapters
│   │       ├── password_hasher.rs  # Argon2 implementation
│   │       ├── token_provider.rs   # Session tokens
│   │       └── mod.rs
│   │
│   └── presentation/               # 🔴 API Layer (Inbound)
│       ├── mod.rs
│       ├── routes.rs               # Route definitions
│       ├── handlers/               # HTTP handlers
│       │   ├── auth_handler.rs
│       │   ├── user_handler.rs
│       │   └── mod.rs
│       ├── middleware/             # Middleware
│       │   ├── auth_middleware.rs
│       │   └── mod.rs
│       └── extractors/             # Axum extractors
│           └── mod.rs
│
├── migrations/                     # Diesel migrations
├── Cargo.toml                      # Dependencies
└── diesel.toml                     # Diesel configuration
```

---

## 🏛️ Capas de la Arquitectura

### 1. 🔴 Presentation Layer (Inbound Adapters)

**Responsabilidad:** Recibir requests HTTP y transformarlos a llamadas de aplicación.

```rust
// src/presentation/handlers/auth_handler.rs
pub async fn login(
    State(container): State<DependencyContainer>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validar request
    request.validate()?;
    
    // Delegar a servicio de aplicación
    let response = container.auth_service.login(request).await?;
    
    Ok(Json(response))
}
```

**Componentes:**
- `routes.rs` - Definición de rutas
- `handlers/` - Controladores HTTP
- `middleware/` - Auth, CORS, Rate Limiting
- `extractors/` - Custom Axum extractors

### 2. 🟢 Application Layer (Use Cases)

**Responsabilidad:** Orquestar la lógica de negocio y coordinar entre capas.

```rust
// src/application/services/auth_service.rs
impl AuthService {
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        // 1. Buscar usuario
        let user = self.user_repo.find_by_email(&request.identifier).await?;
        
        // 2. Verificar password
        self.password_hasher.verify(&request.password, &user.password_hash)?;
        
        // 3. Crear sesión
        let session = self.session_repo.create(user.id, expires_at).await?;
        
        // 4. Retornar respuesta
        Ok(AuthResponse::new(user.into(), session.id, expires_in))
    }
}
```

**Componentes:**
- `services/` - Servicios de aplicación (use cases)
- `dtos/` - Data Transfer Objects
  - `ts_types.rs` - Tipos exportables a TypeScript

### 3. 🟡 Domain Layer (Core Business)

**Responsabilidad:** Contiene la lógica de negocio pura, sin dependencias externas.

```rust
// src/domain/entities/user.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    // ...
}

// Traits (Ports) - contratos que deben implementar los adaptadores
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<User, DomainError>;
    async fn create(&self, user: User) -> Result<User, DomainError>;
}
```

**Componentes:**
- `entities/` - Entidades de negocio
- `errors/` - Errores de dominio
- `value_objects/` - Objetos de valor inmutables
- `services/` - Servicios de dominio (traits/ports)

### 4. 🔵 Infrastructure Layer (Outbound Adapters)

**Responsabilidad:** Implementar las interfaces del dominio con tecnologías específicas.

```rust
// src/infrastructure/persistence/user_repository.rs
pub struct PostgresUserRepository {
    pool: Pool<AsyncPgConnection>,
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        let conn = self.pool.get().await?;
        
        users::table
            .find(id)
            .first::<UserModel>(&conn)
            .await
            .map(|m| m.into())
            .map_err(|e| DomainError::NotFound)
    }
}
```

**Componentes:**
- `persistence/` - Repositorios (Diesel/PostgreSQL)
- `security/` - Hashers, token providers
- `container.rs` - Dependency Injection

---

## 🔄 Flujo de Datos

```
HTTP Request
     │
     ▼
┌─────────────────┐
│  Presentation   │  1. Recibe request HTTP
│   (Handler)     │  2. Valida con Validator
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Application    │  3. Ejecuta caso de uso
│   (Service)     │  4. Coordina repositorios
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Domain       │  5. Aplica reglas de negocio
│  (Entity/Port)  │  6. Valida invariantes
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Infrastructure  │  7. Persiste en DB
│  (Repository)   │  8. Llama servicios externos
└────────┬────────┘
         │
         ▼
   HTTP Response
```

---

## 🔐 Autenticación y Seguridad

### Sistema de Sesiones (NO JWT)

```rust
// Cookie de sesión ultra-segura
Cookie::build(("session_token", token))
    .http_only(true)      // No accesible por JavaScript
    .secure(true)         // Solo HTTPS
    .same_site(Strict)    // Protección CSRF
    .max_age(Duration::hours(24))
    .path("/")
```

### Flujo de Autenticación

```
1. Login Request
   ├─ Validar credenciales (email/password)
   ├─ Verificar password con Argon2id
   ├─ Crear sesión en DB
   └─ Retornar cookie HttpOnly

2. Request Autenticado
   ├─ Extraer token de cookie
   ├─ Hash del token (SHA-256)
   ├─ Buscar sesión activa en DB
   ├─ Verificar expiración
   └─ Injectar user en request

3. Logout
   ├─ Revocar sesión en DB
   └─ Eliminar cookie
```

### Seguridad del Password

```rust
// Argon2id con parámetros seguros
Argon2::new(
    Algorithm::Argon2id,
    Version::V0x13,
    Params::new(
        65536,  // 64 MB memory
        3,      // iterations
        4,      // parallelism
        None,
    )?
)
```

---

## 🔄 Generación de Tipos TypeScript

### Usando ts-rs

Los tipos Rust se exportan automáticamente a TypeScript:

```rust
// src/application/dtos/ts_types.rs
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../frontend/src/types/generated/")]
pub struct UserInfoTs {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRoleTs,
}
```

### Generar Tipos

```bash
# Desde el directorio backend
cargo test export_types -- --nocapture
```

### Resultado en Frontend

```typescript
// frontend/src/types/generated/UserInfoTs.ts
export interface UserInfoTs {
    id: string;       // UUID
    username: string;
    email: string;
    role: UserRoleTs;
}
```

---

## 🌐 API Endpoints

### Auth Endpoints

| Método | Ruta | Descripción |
|--------|------|-------------|
| POST | `/api/v1/auth/login` | Iniciar sesión |
| POST | `/api/v1/auth/register` | Registrar usuario |
| POST | `/api/v1/auth/logout` | Cerrar sesión |
| GET | `/api/v1/auth/me` | Usuario actual |
| PUT | `/api/v1/auth/password` | Cambiar contraseña |

### User Management

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/v1/users` | Listar usuarios |
| GET | `/api/v1/users/:id` | Obtener usuario |
| POST | `/api/v1/users` | Crear usuario |
| PUT | `/api/v1/users/:id` | Actualizar usuario |
| DELETE | `/api/v1/users/:id` | Eliminar usuario |

### TourOperator (Próximos)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/v1/viajes` | Listar viajes |
| POST | `/api/v1/reservaciones` | Crear reservación |
| GET | `/api/v1/pasajeros` | Listar pasajeros |
| GET | `/api/v1/empresas` | Listar empresas |
| GET | `/api/v1/rutas` | Listar rutas |

---

## 🛠️ Guía de Desarrollo

### Agregar Nueva Entidad

1. **Domain**: Crear entidad en `domain/entities/`
2. **Domain**: Definir trait (port) en `domain/services/`
3. **Infrastructure**: Implementar repositorio en `infrastructure/persistence/`
4. **Application**: Crear servicio en `application/services/`
5. **Application**: Crear DTOs en `application/dtos/`
6. **Presentation**: Crear handlers en `presentation/handlers/`
7. **ts-rs**: Agregar tipos a `dtos/ts_types.rs`

### Comandos Útiles

```bash
# Compilar
cargo build --release

# Ejecutar tests
cargo test

# Generar tipos TypeScript
cargo test export_types -- --nocapture

# Ejecutar migraciones
diesel migration run

# Ver logs con nivel debug
RUST_LOG=debug cargo run
```

---

## 📝 Convenciones de Código

1. **Nombres de archivos**: snake_case
2. **Structs/Enums**: PascalCase
3. **Funciones/Variables**: snake_case
4. **Traits**: PascalCase con sufijo descriptivo
5. **DTOs para TS**: Sufijo `Ts` (ej: `UserInfoTs`)

---

*Documentación generada para TourOperator - Sistema de Gestión de Pasajeros*
*Última actualización: Diciembre 2024*
