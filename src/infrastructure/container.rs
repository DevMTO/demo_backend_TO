//! # Dependency Container
//! 
//! Contenedor de inyección de dependencias.

use std::sync::Arc;

use crate::config::AppConfig;
use crate::application::ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    PasswordHasherPort,
    SessionManagerPort,
};
use crate::application::use_cases::auth::{
    LoginUseCase,
    LogoutUseCase,
    VerifySessionUseCase,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::{
    DatabasePool,
    repositories::{PostgresUserRepository, PostgresSessionRepository},
};
use crate::infrastructure::security::{
    Argon2PasswordHasher,
    SecureSessionManager,
};

/// Contenedor de dependencias
/// 
/// Expone solo los componentes necesarios para la capa de presentación.
pub struct DependencyContainer {
    // Use Cases - Auth
    pub login_use_case: Arc<LoginUseCase>,
    pub logout_use_case: Arc<LogoutUseCase>,
    pub verify_session_use_case: Arc<VerifySessionUseCase>,
    
    // Session Manager para middleware
    pub session_manager: Arc<dyn SessionManagerPort>,
    
    // Config
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_http_only: bool,
    pub cookie_max_age_hours: i64,
}

impl DependencyContainer {
    pub fn new(db_pool: DatabasePool, config: AppConfig) -> Result<Self, ApplicationError> {
        // Validar configuración de seguridad
        config.validate_security()
            .map_err(|e| ApplicationError::Configuration(e.to_string()))?;
        
        // Crear repositorios
        let user_repository: Arc<dyn UserRepositoryPort> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );
        let session_repository: Arc<dyn SessionRepositoryPort> = Arc::new(
            PostgresSessionRepository::new(db_pool.clone())
        );
        
        // Crear servicios de seguridad
        let password_hasher: Arc<dyn PasswordHasherPort> = Arc::new(
            Argon2PasswordHasher::with_params(
                config.argon2_memory_size,
                config.argon2_iterations,
                config.argon2_parallelism,
            )?
        );
        
        let session_manager: Arc<dyn SessionManagerPort> = Arc::new(
            SecureSessionManager::new(&config)?
        );
        
        // Crear casos de uso
        let login_use_case = Arc::new(LoginUseCase::new(
            user_repository.clone(),
            session_repository.clone(),
            password_hasher.clone(),
            session_manager.clone(),
        ));
        
        let logout_use_case = Arc::new(LogoutUseCase::new(
            session_repository.clone(),
        ));
        
        let verify_session_use_case = Arc::new(VerifySessionUseCase::new(
            user_repository,
            session_repository,
            session_manager.clone(),
        ));
        
        Ok(Self {
            login_use_case,
            logout_use_case,
            verify_session_use_case,
            session_manager,
            cookie_name: config.cookie_name,
            cookie_secure: config.cookie_secure,
            cookie_same_site: config.cookie_same_site,
            cookie_domain: config.cookie_domain,
            cookie_path: config.cookie_path,
            cookie_http_only: config.cookie_http_only,
            cookie_max_age_hours: config.cookie_max_age_hours,
        })
    }
}
