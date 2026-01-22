//! User Service - Lógica de negocio para usuarios
//! 
//! Este servicio contiene toda la lógica de negocio relacionada con usuarios:
//! - Creación de usuarios (con validaciones de unicidad)
//! - Actualización de usuarios
//! - Activación/Desactivación
//! - Cambio de contraseña por admin
//! - Logging de actividades
//! - Notificaciones

use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn, instrument};

use crate::application::dtos::{
    CreateUserRequest, UpdateUserRequest, UserDetailDto, 
    UserListItemDto, AdminChangePasswordRequest,
};
use crate::application::ports::{
    UserRepositoryPort, PersonaRepositoryPort, PasswordHasherPort,
    NotificationServicePort,
};
use crate::application::services::LoggingService;
use crate::domain::entities::{
    User, UserRole, Persona, TipoDocumento, EntityType,
    NotificationType, NotificationCategory, NotificationPriority,
};
use crate::domain::errors::ApplicationError;

/// Resultado de crear un usuario con persona opcional
pub struct CreateUserResult {
    pub user: UserDetailDto,
}

/// Resultado de actualizar un usuario
pub struct UpdateUserResult {
    pub user: UserDetailDto,
}

/// Servicio de usuarios - contiene la lógica de negocio
pub struct UserService {
    user_repository: Arc<dyn UserRepositoryPort>,
    persona_repository: Arc<dyn PersonaRepositoryPort>,
    password_hasher: Arc<dyn PasswordHasherPort>,
    logging_service: Arc<LoggingService>,
    notification_service: Arc<dyn NotificationServicePort>,
}

impl UserService {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        persona_repository: Arc<dyn PersonaRepositoryPort>,
        password_hasher: Arc<dyn PasswordHasherPort>,
        logging_service: Arc<LoggingService>,
        notification_service: Arc<dyn NotificationServicePort>,
    ) -> Self {
        Self {
            user_repository,
            persona_repository,
            password_hasher,
            logging_service,
            notification_service,
        }
    }

    /// Listar usuarios con paginación
    #[instrument(skip(self))]
    pub async fn list_users(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<UserListItemDto>, i64), ApplicationError> {
        let page_size = page_size.min(100).max(1);
        let offset = (page - 1).max(0) * page_size;
        
        self.user_repository
            .list_users_with_details(page_size, offset)
            .await
    }

    /// Obtener un usuario por ID
    #[instrument(skip(self))]
    pub async fn get_user(&self, id: i32) -> Result<UserDetailDto, ApplicationError> {
        let user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        Ok(UserDetailDto::from(user))
    }

    /// Crear un nuevo usuario
    /// 
    /// # Validaciones de negocio:
    /// - Email debe ser único
    /// - Username debe ser único
    /// - Si se proporciona id_persona, debe existir
    /// - Si se proporciona nueva_persona, el documento debe ser único
    #[instrument(skip(self, request))]
    pub async fn create_user(
        &self,
        request: CreateUserRequest,
        created_by: i32,
        created_by_username: Option<String>,
    ) -> Result<CreateUserResult, ApplicationError> {
        // Validar unicidad de email
        if self.user_repository.exists_by_email(&request.email).await? {
            return Err(ApplicationError::Conflict(format!("Email {} ya registrado", request.email)));
        }
        
        // Validar unicidad de username
        if self.user_repository.exists_by_username(&request.username).await? {
            return Err(ApplicationError::Conflict(format!("Username {} ya existe", request.username)));
        }
        
        // Determinar id_persona
        let (id_persona, _persona_created) = self.resolve_persona(&request, created_by).await?;
        
        // Hash de la contraseña
        let password_hash = self.password_hasher.hash(&request.password)?;
        
        // Parsear el rol
        let role = request.role.parse::<UserRole>()
            .map_err(|_| ApplicationError::Validation(format!("Rol inválido: {}", request.role)))?;
        
        // Crear la entidad User
        let now = Utc::now();
        let new_user = User {
            id: 0,
            id_persona,
            username: request.username.clone(),
            email: request.email.to_lowercase(),
            password_hash,
            role: role.clone(),
            id_entidad: request.id_entidad,
            is_active: true,
            last_login: None,
            created_at: now,
            updated_at: now,
            created_by: Some(created_by),
            updated_by: Some(created_by),
        };
        
        let created = self.user_repository.create(&new_user).await?;
        info!("Usuario creado: {} (ID: {})", created.username, created.id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_create::<User>(
            Some(created_by),
            created_by_username,
            EntityType::User,
            created.id,
            &created.username,
            Some(&created),
            None,
        ).await {
            warn!("Error al registrar log de creación de usuario: {}", e);
        }
        
        // Notificación a admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Nuevo usuario creado",
            &format!("Se ha creado el usuario '{}' con rol {}", created.username, role),
            NotificationType::Info,
            NotificationCategory::Crud,
            NotificationPriority::Low,
            Some(created_by),
        ).await {
            warn!("Error al enviar notificación de usuario creado: {}", e);
        }
        
        Ok(CreateUserResult {
            user: UserDetailDto::from(created),
        })
    }

    /// Resolver persona para un nuevo usuario
    async fn resolve_persona(
        &self,
        request: &CreateUserRequest,
        created_by: i32,
    ) -> Result<(Option<i32>, Option<i32>), ApplicationError> {
        if let Some(id) = request.id_persona {
            // Verificar que la persona existe
            let _persona = self.persona_repository
                .find_by_id(id)
                .await?
                .ok_or_else(|| ApplicationError::NotFound(format!("Persona {} no encontrada", id)))?;
            Ok((Some(id), None))
        } else if let Some(ref nueva_persona) = request.nueva_persona {
            // Verificar que el documento no exista
            if self.persona_repository.exists_by_documento(&nueva_persona.tipo_documento, &nueva_persona.nro_documento).await? {
                return Err(ApplicationError::Conflict(format!("Documento {} ya registrado", nueva_persona.nro_documento)));
            }
            
            // Crear la persona
            let now = Utc::now();
            let tipo_doc = nueva_persona.tipo_documento.parse::<TipoDocumento>()
                .unwrap_or(TipoDocumento::Dni);
            
            let persona = Persona {
                id: 0,
                tipo_documento: tipo_doc,
                nro_documento: nueva_persona.nro_documento.clone(),
                nombre: nueva_persona.nombre.clone(),
                apellidos: nueva_persona.apellidos.clone(),
                telefono: nueva_persona.telefono.clone(),
                correo: Some(request.email.clone()),
                fecha_nacimiento: nueva_persona.fecha_nacimiento,
                created_at: now,
                updated_at: now,
                created_by: Some(created_by),
                updated_by: Some(created_by),
            };
            
            let created_persona = self.persona_repository.create(&persona).await?;
            info!("Persona creada: {} {} (ID: {})", created_persona.nombre, created_persona.apellidos, created_persona.id);
            Ok((Some(created_persona.id), Some(created_persona.id)))
        } else {
            Ok((None, None))
        }
    }

    /// Actualizar un usuario existente
    /// 
    /// # Validaciones de negocio:
    /// - Usuario debe existir
    /// - Si se cambia el email, el nuevo email debe ser único
    #[instrument(skip(self, request))]
    pub async fn update_user(
        &self,
        id: i32,
        request: UpdateUserRequest,
        updated_by: i32,
        updated_by_username: Option<String>,
    ) -> Result<UpdateUserResult, ApplicationError> {
        // Buscar usuario existente
        let user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        // Verificar unicidad de email si se está cambiando
        if let Some(ref new_email) = request.email {
            let email_lower = new_email.to_lowercase();
            if email_lower != user.email {
                if self.user_repository.exists_by_email(&email_lower).await? {
                    return Err(ApplicationError::Conflict(format!("Email {} ya registrado", new_email)));
                }
            }
        }
        
        // Guardar estado anterior para comparación
        let old_user = user.clone();
        
        // Aplicar cambios
        let updated = request.apply_to(user, Some(updated_by));
        let result = self.user_repository.update(&updated).await?;
        
        info!("Usuario actualizado: {} (ID: {})", result.username, result.id);
        
        // Detectar campos cambiados
        let changed_fields = self.detect_changed_fields(&old_user, &result);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<User>(
            Some(updated_by),
            updated_by_username,
            EntityType::User,
            id,
            Some(&old_user),
            Some(&result),
            if changed_fields.is_empty() { None } else { Some(changed_fields.clone()) },
            None,
        ).await {
            warn!("Error al registrar log de actualización de usuario: {}", e);
        }
        
        // Notificación al usuario afectado si fue actualizado por otro
        if id != updated_by {
            let notification_msg = if changed_fields.is_empty() {
                "Tu cuenta ha sido actualizada".to_string()
            } else {
                format!("Se actualizaron los siguientes campos de tu cuenta: {}", changed_fields.join(", "))
            };
            
            if let Err(e) = self.notification_service.notify_user(
                id,
                "Cuenta actualizada",
                &notification_msg,
                NotificationType::Info,
                NotificationCategory::Crud,
                NotificationPriority::Normal,
                Some(updated_by),
            ).await {
                warn!("Error al enviar notificación de actualización: {}", e);
            }
        }
        
        Ok(UpdateUserResult {
            user: UserDetailDto::from(result),
        })
    }

    /// Detectar campos que cambiaron entre dos estados de usuario
    fn detect_changed_fields(&self, old: &User, new: &User) -> Vec<String> {
        let mut changed = Vec::new();
        if old.email != new.email { changed.push("email".to_string()); }
        if old.role != new.role { changed.push("role".to_string()); }
        if old.is_active != new.is_active { changed.push("is_active".to_string()); }
        if old.id_entidad != new.id_entidad { changed.push("id_entidad".to_string()); }
        if old.id_persona != new.id_persona { changed.push("id_persona".to_string()); }
        if old.username != new.username { changed.push("username".to_string()); }
        changed
    }

    /// Eliminar (soft delete) un usuario
    /// 
    /// # Validaciones de negocio:
    /// - Usuario debe existir
    /// - No se puede eliminar a uno mismo
    #[instrument(skip(self))]
    pub async fn delete_user(
        &self,
        id: i32,
        deleted_by: i32,
        deleted_by_username: Option<String>,
    ) -> Result<User, ApplicationError> {
        let user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        // No permitir eliminar el propio usuario
        if user.id == deleted_by {
            return Err(ApplicationError::Forbidden("No puedes desactivar tu propio usuario".to_string()));
        }
        
        self.user_repository.delete(id).await?;
        info!("🗑️ Usuario desactivado: {} (ID: {})", user.username, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_delete::<User>(
            Some(deleted_by),
            deleted_by_username.clone(),
            EntityType::User,
            id,
            Some(&user),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de usuario: {}", e);
        }
        
        // Notificación al usuario desactivado
        if let Err(e) = self.notification_service.notify_user(
            id,
            "Cuenta desactivada",
            "Tu cuenta ha sido desactivada por un administrador. Contacta con soporte si crees que es un error.",
            NotificationType::Warning,
            NotificationCategory::Auth,
            NotificationPriority::High,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación de desactivación: {}", e);
        }
        
        // Notificación a admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Usuario desactivado",
            &format!("El usuario '{}' ha sido desactivado por {}", user.username, deleted_by_username.as_deref().unwrap_or("sistema")),
            NotificationType::Warning,
            NotificationCategory::Auth,
            NotificationPriority::Normal,
            Some(deleted_by),
        ).await {
            warn!("Error al enviar notificación a admins: {}", e);
        }
        
        Ok(user)
    }

    /// Activar un usuario
    /// 
    /// # Validaciones de negocio:
    /// - Usuario debe existir
    /// - Usuario no debe estar ya activo
    #[instrument(skip(self))]
    pub async fn activate_user(
        &self,
        id: i32,
        activated_by: i32,
        activated_by_username: Option<String>,
    ) -> Result<(UserDetailDto, bool), ApplicationError> {
        let mut user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        if user.is_active {
            return Err(ApplicationError::Conflict("El usuario ya está activo".to_string()));
        }
        
        let old_active = user.is_active;
        user.is_active = true;
        user.updated_at = Utc::now();
        user.updated_by = Some(activated_by);
        
        let result = self.user_repository.update(&user).await?;
        info!("Usuario activado: {} (ID: {})", result.username, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<User>(
            Some(activated_by),
            activated_by_username.clone(),
            EntityType::User,
            id,
            None::<&User>,
            None::<&User>,
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de activación de usuario: {}", e);
        }
        
        // Notificación al usuario activado
        if let Err(e) = self.notification_service.notify_user(
            id,
            "Cuenta activada",
            "Tu cuenta ha sido activada nuevamente. Ya puedes iniciar sesión.",
            NotificationType::Success,
            NotificationCategory::Auth,
            NotificationPriority::High,
            Some(activated_by),
        ).await {
            warn!("Error al enviar notificación de activación: {}", e);
        }
        
        // Notificación a admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Usuario activado",
            &format!("El usuario '{}' ha sido activado por {} (estado anterior: {})", 
                result.username, 
                activated_by_username.as_deref().unwrap_or("sistema"),
                if old_active { "activo" } else { "inactivo" }
            ),
            NotificationType::Info,
            NotificationCategory::Auth,
            NotificationPriority::Low,
            Some(activated_by),
        ).await {
            warn!("Error al enviar notificación a admins: {}", e);
        }
        
        Ok((UserDetailDto::from(result), old_active))
    }

    /// Desactivar un usuario
    /// 
    /// # Validaciones de negocio:
    /// - Usuario debe existir
    /// - No se puede desactivar a uno mismo
    /// - Usuario no debe estar ya inactivo
    #[instrument(skip(self))]
    pub async fn deactivate_user(
        &self,
        id: i32,
        deactivated_by: i32,
        deactivated_by_username: Option<String>,
    ) -> Result<(UserDetailDto, bool), ApplicationError> {
        let mut user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        // No permitir desactivar el propio usuario
        if user.id == deactivated_by {
            return Err(ApplicationError::Forbidden("No puedes desactivar tu propio usuario".to_string()));
        }
        
        if !user.is_active {
            return Err(ApplicationError::Conflict("El usuario ya está desactivado".to_string()));
        }
        
        let old_active = user.is_active;
        user.is_active = false;
        user.updated_at = Utc::now();
        user.updated_by = Some(deactivated_by);
        
        let result = self.user_repository.update(&user).await?;
        info!("Usuario desactivado: {} (ID: {})", result.username, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<User>(
            Some(deactivated_by),
            deactivated_by_username.clone(),
            EntityType::User,
            id,
            None::<&User>,
            None::<&User>,
            Some(vec!["is_active".to_string()]),
            None,
        ).await {
            warn!("Error al registrar log de desactivación de usuario: {}", e);
        }
        
        // Notificación al usuario desactivado
        if let Err(e) = self.notification_service.notify_user(
            id,
            "Cuenta desactivada",
            "Tu cuenta ha sido desactivada por un administrador. Contacta con soporte si crees que es un error.",
            NotificationType::Warning,
            NotificationCategory::Auth,
            NotificationPriority::High,
            Some(deactivated_by),
        ).await {
            warn!("Error al enviar notificación de desactivación: {}", e);
        }
        
        // Notificación a admins
        if let Err(e) = self.notification_service.notify_roles(
            vec![UserRole::SuperAdmin, UserRole::Admin],
            "Usuario desactivado manualmente",
            &format!("El usuario '{}' ha sido desactivado manualmente por {} (estado anterior: {})", 
                result.username, 
                deactivated_by_username.as_deref().unwrap_or("sistema"),
                if old_active { "activo" } else { "inactivo" }
            ),
            NotificationType::Warning,
            NotificationCategory::Auth,
            NotificationPriority::Normal,
            Some(deactivated_by),
        ).await {
            warn!("Error al enviar notificación a admins: {}", e);
        }
        
        Ok((UserDetailDto::from(result), old_active))
    }

    /// Cambiar contraseña de un usuario (operación administrativa)
    /// 
    /// # Validaciones de negocio:
    /// - Usuario debe existir
    #[instrument(skip(self, request))]
    pub async fn admin_change_password(
        &self,
        id: i32,
        request: AdminChangePasswordRequest,
        changed_by: i32,
        changed_by_username: Option<String>,
    ) -> Result<UserDetailDto, ApplicationError> {
        let mut user = self.user_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Usuario {} no encontrado", id)))?;
        
        // Hash de la nueva contraseña
        let new_password_hash = self.password_hasher.hash(&request.new_password)?;
        
        // Actualizar contraseña
        user.update_password(new_password_hash);
        user.updated_by = Some(changed_by);
        
        let result = self.user_repository.update(&user).await?;
        info!("Contraseña cambiada para usuario: {} (ID: {})", result.username, id);
        
        // Logging del evento
        if let Err(e) = self.logging_service.log_update::<User>(
            Some(changed_by),
            changed_by_username,
            EntityType::User,
            id,
            None::<&User>,
            None::<&User>,
            Some(vec!["password".to_string()]),
            Some("Contraseña cambiada por SuperAdmin".to_string()),
        ).await {
            warn!("Error al registrar log de cambio de contraseña: {}", e);
        }
        
        // Notificación al usuario afectado
        if let Err(e) = self.notification_service.notify_user(
            id,
            "Contraseña actualizada",
            "Tu contraseña ha sido actualizada por un administrador. Si no solicitaste este cambio, contacta con soporte.",
            NotificationType::Warning,
            NotificationCategory::Auth,
            NotificationPriority::High,
            Some(changed_by),
        ).await {
            warn!("Error al enviar notificación de cambio de contraseña: {}", e);
        }
        
        Ok(UserDetailDto::from(result))
    }
}
