//! Creación de componentes de autenticación y seguridad

use std::sync::Arc;

use crate::application::ports::{PasswordHasherPort, SessionManagerPort, UserRepositoryPort, SessionRepositoryPort};
use crate::application::use_cases::auth::{LoginUseCase, LogoutUseCase, VerifySessionUseCase};
use crate::config::AppConfig;
use crate::domain::errors::ApplicationError;
use crate::infrastructure::security::{Argon2PasswordHasher, SecureSessionManager};

/// Componentes de autenticación y seguridad
pub(super) struct AuthComponents {
    pub login_use_case: Arc<LoginUseCase>,
    pub logout_use_case: Arc<LogoutUseCase>,
    pub verify_session_use_case: Arc<VerifySessionUseCase>,
    pub session_manager: Arc<dyn SessionManagerPort>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
}

impl AuthComponents {
    pub fn create(
        config: &AppConfig,
        user_repository: Arc<dyn UserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
    ) -> Result<Self, ApplicationError> {
        let password_hasher: Arc<dyn PasswordHasherPort> = Arc::new(
            Argon2PasswordHasher::with_params(
                config.argon2_memory_size,
                config.argon2_iterations,
                config.argon2_parallelism,
            )?
        );

        let session_manager: Arc<dyn SessionManagerPort> = Arc::new(
            SecureSessionManager::new(config)?
        );

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
            password_hasher,
        })
    }
}
