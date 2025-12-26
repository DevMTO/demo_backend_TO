//! # Login Use Case
//! 
//! Caso de uso para iniciar sesión con cookies de sesión.

use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};

use crate::domain::{
    entities::UserSession,
    errors::ApplicationError,
};
use crate::application::ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    PasswordHasherPort,
    SessionManagerPort,
};
use crate::application::dtos::auth_dto::{LoginRequest, AuthUserInfo};

/// Resultado del login (para cookies de sesión)
pub struct LoginOutput {
    pub user_info: AuthUserInfo,
    pub session_id: Uuid,
    pub session_token: String,
    pub expires_in_seconds: i64,
}

/// Use case para login con sesiones seguras
pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    password_hasher: Arc<dyn PasswordHasherPort>,
    session_manager: Arc<dyn SessionManagerPort>,
    max_sessions: i64,
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
            max_sessions: 5,
        }
    }
    
    /// Ejecutar el caso de uso de login
    #[instrument(skip(self, request, ip_address, user_agent), fields(identifier = %request.identifier))]
    pub async fn execute(
        &self,
        request: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginOutput, ApplicationError> {
        debug!("📝 Buscando usuario por email/username: {}", request.identifier);
        
        // 1. Buscar usuario por email o username
        let user = match self.user_repository
            .find_by_email_or_username(&request.identifier)
            .await {
                Ok(Some(user)) => {
                    info!("👤 Usuario encontrado: {} (id: {})", user.username, user.id);
                    user
                },
                Ok(None) => {
                    warn!("❌ Usuario no encontrado: {}", request.identifier);
                    return Err(ApplicationError::Authentication("Credenciales inválidas".to_string()));
                },
                Err(e) => {
                    warn!("❌ Error al buscar usuario: {:?}", e);
                    return Err(e);
                }
            };
        
        // 2. Verificar que el usuario esté activo
        debug!("🔍 Verificando estado del usuario: {:?}", user.status);
        if !user.is_active() {
            warn!("❌ Usuario inactivo: {} (status: {:?})", user.username, user.status);
            return Err(ApplicationError::Authentication(
                "Usuario inactivo".to_string()
            ));
        }
        
        // 3. Verificar contraseña
        debug!("🔐 Verificando contraseña...");
        let password_valid = match self.password_hasher.verify(&request.password, &user.password_hash) {
            Ok(valid) => valid,
            Err(e) => {
                warn!("❌ Error al verificar contraseña: {:?}", e);
                return Err(e);
            }
        };
        
        if !password_valid {
            warn!("❌ Contraseña incorrecta para usuario: {}", user.username);
            return Err(ApplicationError::Authentication(
                "Credenciales inválidas".to_string()
            ));
        }
        
        info!("✅ Contraseña válida para: {}", user.username);
        
        // 4. Verificar límite de sesiones activas
        debug!("📊 Verificando sesiones activas...");
        let active_sessions_count = self.session_repository
            .count_active_by_user_id(&user.id)
            .await?;
        
        debug!("Sesiones activas: {}/{}", active_sessions_count, self.max_sessions);
        
        if active_sessions_count >= self.max_sessions {
            info!("⚠️ Límite de sesiones alcanzado, revocando sesión más antigua");
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
        
        // 5. Crear sesión con token opaco
        debug!("🎫 Creando nueva sesión...");
        let (session, token_data) = self.session_manager.create_session(
            user.id,
            user_agent,
            ip_address,
        )?;
        
        // 6. Guardar sesión en BD
        debug!("💾 Guardando sesión en BD...");
        let created_session = self.session_repository.create(&session).await?;
        info!("✅ Sesión creada: {} (expira: {})", created_session.id, created_session.expires_at);
        
        // 7. Actualizar último login del usuario
        debug!("📅 Actualizando último login...");
        let mut updated_user = user.clone();
        updated_user.update_last_login();
        self.user_repository.update(&updated_user).await?;
        
        // 8. Construir respuesta
        let user_info = AuthUserInfo {
            id: user.id,
            id_persona: user.id_persona,
            username: user.username,
            email: user.email,
            role: user.role.to_string(),
            id_entidad: user.id_entidad,
            nombre_entidad: user.nombre_entidad,
            status: user.status.to_string(),
        };
        
        let expires_in = created_session.expires_at.timestamp() - chrono::Utc::now().timestamp();
        
        info!("🎉 Login completado exitosamente para: {} (sesión expira en {} segundos)", 
            user_info.username, expires_in);
        
        Ok(LoginOutput {
            user_info,
            session_id: created_session.id,
            session_token: token_data.token,
            expires_in_seconds: expires_in,
        })
    }
}
