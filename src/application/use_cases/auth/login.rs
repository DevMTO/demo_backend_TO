//! # Login Use Case
//! 
//! Caso de uso para iniciar sesión con cookies ultra-seguras (NO JWT).


use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    entities::UserSession,
    errors::ApplicationError,
    services::SessionPolicy,
};
use crate::application::ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    PasswordHasherPort,
    SessionManagerPort,
};
use crate::application::dtos::{LoginRequest, UserInfo};

/// Resultado del login (para cookies de sesión)
pub struct LoginOutput {
    pub user_info: UserInfo,
    pub session_id: Uuid,
    pub session_token: String,  // Token plano para la cookie
    pub expires_in_seconds: i64,
    pub session: UserSession,
}

/// Use case para login con sesiones seguras
pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    password_hasher: Arc<dyn PasswordHasherPort>,
    session_manager: Arc<dyn SessionManagerPort>,
    session_policy: SessionPolicy,
}

impl LoginUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
        password_hasher: Arc<dyn PasswordHasherPort>,
        session_manager: Arc<dyn SessionManagerPort>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            password_hasher,
            session_manager,
            session_policy: SessionPolicy::default(),
        }
    }
    
    pub fn with_policy(mut self, policy: SessionPolicy) -> Self {
        self.session_policy = policy;
        self
    }
    
    /// Ejecutar el caso de uso de login
    pub async fn execute(
        &self,
        request: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginOutput, ApplicationError> {
        // 1. Buscar usuario por email o username
        let user = self.user_repository
            .find_by_email_or_username(&request.identifier)
            .await?
            .ok_or_else(|| {
                ApplicationError::Authentication("Credenciales inválidas".to_string())
            })?;
        
        // 2. Verificar que el usuario esté activo
        if !user.is_active {
            return Err(ApplicationError::Authentication(
                "Usuario inactivo".to_string()
            ));
        }
        
        // 3. Verificar contraseña
        let password_valid = self.password_hasher.verify(&request.password, &user.password_hash)?;
        if !password_valid {
            return Err(ApplicationError::Authentication(
                "Credenciales inválidas".to_string()
            ));
        }
        
        // 4. Verificar MFA si está habilitado
        if user.mfa_enabled {
            match &request.mfa_code {
                Some(_code) => {
                    // TODO: Implementar verificación MFA (TOTP)
                }
                None => {
                    return Err(ApplicationError::Domain(
                        crate::domain::errors::DomainError::MfaRequired
                    ));
                }
            }
        }
        
        // 5. Verificar límite de sesiones activas
        let active_sessions_count = self.session_repository
            .count_active_by_user_id(&user.id)
            .await?;
        
        if active_sessions_count >= self.session_policy.max_sessions_per_user as i64 {
            // Revocar la sesión más antigua si se excede el límite
            let sessions = self.session_repository
                .find_active_by_user_id(&user.id)
                .await?;
            
            if let Some(oldest) = sessions.first() {
                self.session_repository
                    .revoke(&oldest.id, "Límite de sesiones excedido")
                    .await?;
            }
        }
        
        // 6. Crear sesión con token opaco (NO JWT)
        let (session, token_data) = self.session_manager.create_session(
            user.id,
            user_agent,
            ip_address,
        )?;
        
        // 7. Guardar sesión en BD
        let created_session = self.session_repository.create(&session).await?;
        
        // 8. Actualizar último login del usuario
        let mut updated_user = user.clone();
        updated_user.update_last_login();
        self.user_repository.update(&updated_user).await?;
        
        // 9. Construir respuesta
        let user_info = UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role.to_string(),
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
        };
        
        let expires_in = created_session.expires_at.timestamp() - chrono::Utc::now().timestamp();
        
        Ok(LoginOutput {
            user_info,
            session_id: created_session.id,
            session_token: token_data.token,  // Token plano para la cookie
            expires_in_seconds: expires_in,
            session: created_session,
        })
    }
}
