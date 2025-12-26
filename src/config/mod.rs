//! # Application Configuration
//! 
//! Configuración centralizada de la aplicación con sesiones ultra-seguras.

use std::env;

/// Configuración principal de la aplicación
#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,
    
    // Database (Async con Deadpool)
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_connection_timeout: u64,
    
    // Session Configuration (Cookies Ultra-Seguras, SIN JWT)
    pub session_secret: String,
    pub session_expiration_hours: i64,
    pub session_idle_timeout_minutes: i64,
    pub session_rotation_interval_minutes: i64,
    
    // Cookie Settings
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_http_only: bool,
    pub cookie_max_age_hours: i64,
    
    // CORS
    pub cors_allowed_origins: Vec<String>,
    pub cors_max_age_secs: u64,
    pub cors_allow_credentials: bool,
    
    // Security - Argon2
    pub argon2_memory_size: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
    
    // Rate Limiting (Governor)
    pub rate_limit_requests_per_second: u32,
    pub rate_limit_burst_size: u32,
    
    // Brute Force Protection
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    
    // Request Limits
    pub max_request_body_size: usize,
    pub request_timeout_secs: u64,
}

impl AppConfig {
    /// Crear configuración desde variables de entorno
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            // Server
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            
            // Database (Async)
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            database_min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
            database_connection_timeout: env::var("DATABASE_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            
            // Session (Ultra-Secure)
            session_secret: env::var("SESSION_SECRET")
                .map_err(|_| anyhow::anyhow!("SESSION_SECRET must be set (min 64 chars)"))?,
            session_expiration_hours: env::var("SESSION_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            session_idle_timeout_minutes: env::var("SESSION_IDLE_TIMEOUT_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            session_rotation_interval_minutes: env::var("SESSION_ROTATION_INTERVAL_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            
            // Cookie Settings
            cookie_name: env::var("COOKIE_NAME")
                .unwrap_or_else(|_| "__Secure-SessionId".to_string()),
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            cookie_same_site: env::var("COOKIE_SAME_SITE")
                .unwrap_or_else(|_| "strict".to_string()),
            cookie_domain: env::var("COOKIE_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_string()),
            cookie_path: env::var("COOKIE_PATH")
                .unwrap_or_else(|_| "/".to_string()),
            cookie_http_only: env::var("COOKIE_HTTP_ONLY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cookie_max_age_hours: env::var("COOKIE_MAX_AGE_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            
            // CORS
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            cors_max_age_secs: env::var("CORS_MAX_AGE_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            cors_allow_credentials: env::var("CORS_ALLOW_CREDENTIALS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            
            // Security - Argon2
            argon2_memory_size: env::var("ARGON2_MEMORY_SIZE")
                .unwrap_or_else(|_| "65536".to_string())
                .parse()
                .unwrap_or(65536),
            argon2_iterations: env::var("ARGON2_ITERATIONS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            argon2_parallelism: env::var("ARGON2_PARALLELISM")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            
            // Rate Limiting
            rate_limit_requests_per_second: env::var("RATE_LIMIT_REQUESTS_PER_SECOND")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            rate_limit_burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            
            // Brute Force Protection
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            
            // Request Limits
            max_request_body_size: env::var("MAX_REQUEST_BODY_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .unwrap_or(1048576), // 1MB
            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        })
    }
    
    /// Verificar si estamos en modo producción
    pub fn is_production(&self) -> bool {
        self.cookie_secure
    }
    
    /// Validar configuración de seguridad
    pub fn validate_security(&self) -> anyhow::Result<()> {
        if self.session_secret.len() < 64 {
            return Err(anyhow::anyhow!(
                "SESSION_SECRET must be at least 64 characters"
            ));
        }
        Ok(())
    }
}
