//! # Dependency Container
//! 
//! Contenedor de inyección de dependencias con sesiones ultra-seguras.


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
    RegisterUseCase,
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
pub struct DependencyContainer {
    // Repositories
    pub user_repository: Arc<dyn UserRepositoryPort>,
    pub session_repository: Arc<dyn SessionRepositoryPort>,
    
    // Security
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    pub session_manager: Arc<dyn SessionManagerPort>,
    
    // Use Cases
    pub login_use_case: Arc<LoginUseCase>,
    pub register_use_case: Arc<RegisterUseCase>,
    pub logout_use_case: Arc<LogoutUseCase>,
    pub verify_session_use_case: Arc<VerifySessionUseCase>,
    
    // Config
    pub config: AppConfig,
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
        
        let register_use_case = Arc::new(RegisterUseCase::new(
            user_repository.clone(),
            password_hasher.clone(),
        ));
        
        let logout_use_case = Arc::new(LogoutUseCase::new(
            session_repository.clone(),
        ));
        
        let verify_session_use_case = Arc::new(VerifySessionUseCase::new(
            user_repository.clone(),
            session_repository.clone(),
            session_manager.clone(),
        ));
        
        Ok(Self {
            user_repository,
            session_repository,
            password_hasher,
            session_manager,
            login_use_case,
            register_use_case,
            logout_use_case,
            verify_session_use_case,
            config,
        })
    }
}
