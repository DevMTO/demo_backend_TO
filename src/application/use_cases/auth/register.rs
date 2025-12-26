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
use crate::application::dtos::{RegisterRequest, UserDetailDto};

/// Resultado del registro
pub struct RegisterOutput {
    pub user: UserDetailDto,
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
        
        // 5. Parsear el rol (por defecto Operador)
        let role = request.role
            .as_deref()
            .unwrap_or("operador")
            .parse::<UserRole>()
            .unwrap_or_default();
        
        // 6. Crear el usuario
        let user = User::new(
            request.id_persona,
            request.username.clone(),
            request.email.clone(),
            password_hash,
            role,
        );
        
        // 7. Persistir el usuario
        let created_user = self.user_repository.create(&user).await?;
        
        // 8. Construir respuesta
        Ok(RegisterOutput { 
            user: UserDetailDto::from(created_user) 
        })
    }
}
