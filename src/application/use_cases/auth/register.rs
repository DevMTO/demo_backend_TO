//! # Register Use Case
//! 
//! Caso de uso para registro de nuevos usuarios.

use std::sync::Arc;

use crate::domain::{
    entities::{User, UserRole},
    errors::ApplicationError,
};
use crate::application::ports::{
    UserRepositoryPort,
    PasswordHasherPort,
};
use crate::application::dtos::{RegisterRequest, UserInfo};

/// Resultado del registro
pub struct RegisterOutput {
    pub user_info: UserInfo,
}

/// Use case para registro
pub struct RegisterUseCase {
    user_repository: Arc<dyn UserRepositoryPort>,
    password_hasher: Arc<dyn PasswordHasherPort>,
}

impl RegisterUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        password_hasher: Arc<dyn PasswordHasherPort>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
        }
    }
    
    /// Ejecutar el caso de uso de registro
    pub async fn execute(&self, request: RegisterRequest) -> Result<RegisterOutput, ApplicationError> {
        // 1. Validar que las contraseñas coincidan
        if request.password != request.password_confirm {
            return Err(ApplicationError::Validation(
                "Las contraseñas no coinciden".to_string()
            ));
        }
        
        // 2. Verificar que el email no exista
        if self.user_repository.exists_by_email(&request.email).await? {
            return Err(ApplicationError::Conflict(
                "El email ya está registrado".to_string()
            ));
        }
        
        // 3. Verificar que el username no exista
        if self.user_repository.exists_by_username(&request.username).await? {
            return Err(ApplicationError::Conflict(
                "El nombre de usuario ya está en uso".to_string()
            ));
        }
        
        // 4. Hashear la contraseña
        let password_hash = self.password_hasher.hash(&request.password)?;
        
        // 5. Crear el usuario
        let mut user = User::new(
            request.username.clone(),
            request.email.clone(),
            password_hash,
            UserRole::User, // Por defecto, rol de usuario normal
        );
        
        if let Some(display_name) = request.display_name {
            user.display_name = Some(display_name);
        }
        
        // 6. Persistir el usuario
        let created_user = self.user_repository.create(&user).await?;
        
        // 7. TODO: Si se proporcionó documento, crear el registro
        // if let Some(document) = request.document {
        //     // Crear user_document
        // }
        
        // 8. Construir respuesta
        let user_info = UserInfo {
            id: created_user.id,
            username: created_user.username,
            email: created_user.email,
            display_name: created_user.display_name,
            role: created_user.role.to_string(),
            email_verified: created_user.email_verified,
            mfa_enabled: created_user.mfa_enabled,
        };
        
        Ok(RegisterOutput { user_info })
    }
}
