//! # Verify Session Use Case
//! 
//! Caso de uso para verificar sesiones activas usando tokens opacos.

use std::sync::Arc;

use crate::domain::{
    entities::User,
    errors::ApplicationError,
};
use crate::application::ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    SessionManagerPort,
};
use crate::application::dtos::auth_dto::AuthUserInfo;

/// Resultado de verificación de sesión
pub struct SessionVerification {
    pub user: User,
    pub user_info: AuthUserInfo,
    pub session_id: i32,
    /// Nuevo token si fue rotado
    pub new_token: Option<String>,
}

/// Use case para verificar sesión con cookies seguras
pub struct VerifySessionUseCase {
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    session_manager: Arc<dyn SessionManagerPort>,
}

impl VerifySessionUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
        session_manager: Arc<dyn SessionManagerPort>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            session_manager,
        }
    }
    
    /// Verificar un token de sesión (de la cookie)
    pub async fn execute(&self, session_token: &str) -> Result<SessionVerification, ApplicationError> {
        // 1. Calcular hash del token
        let token_hash = self.session_manager.hash_token(session_token)?;
        
        // 2. Buscar la sesión por el hash del token
        let mut session = self.session_repository
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| ApplicationError::SessionRequired)?;
        
        // 3. Verificar que la sesión sea válida (activa, no expirada, no idle)
        self.session_manager.validate_session(&session)?;
        
        // 4. Obtener el usuario
        let user = self.user_repository
            .find_by_id(session.user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("User not found".to_string()))?;
        
        // 5. Verificar que el usuario esté activo
        if !user.is_active() {
            return Err(ApplicationError::Authentication("User is inactive".to_string()));
        }
        
        // 6. Rotar token si es necesario (para mayor seguridad)
        let new_token = if self.session_manager.should_rotate_token(&session) {
            let token_data = self.session_manager.rotate_token(&mut session)?;
            self.session_repository.update(&session).await?;
            Some(token_data.token)
        } else {
            // Solo actualizar última actividad
            self.session_manager.touch_session(&mut session);
            self.session_repository.update(&session).await?;
            None
        };
        
        // 7. Construir respuesta
        let user_info = AuthUserInfo {
            id: user.id,
            id_persona: user.id_persona,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.to_string(),
            id_entidad: user.id_entidad,
            nombre_entidad: user.nombre_entidad.clone(),
            status: user.status.to_string(),
        };
        
        Ok(SessionVerification {
            user,
            user_info,
            session_id: session.id,
            new_token,
        })
    }
}
