use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, info, instrument};

use crate::application::ports::UserRepositoryPort;
use crate::domain::{entities::User, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{NewUserModel, UserModel},
    schema::users,
};

pub struct PostgresUserRepository {
    pool: DatabasePool,
}

impl PostgresUserRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryPort for PostgresUserRepository {
    #[instrument(skip(self, user))]
    async fn create(&self, user: &User) -> Result<User, ApplicationError> {
        debug!("📝 Creando usuario: {}", user.username);
        let mut conn = self.pool.get_connection().await?;
        let new_user: NewUserModel = user.into();
        
        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("❌ Error al crear usuario: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("✅ Usuario creado: {} (id: {})", result.username, result.id);
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, ApplicationError> {
        debug!("🔍 Buscando usuario por ID: {}", id);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::id.eq(id))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("❌ Error al buscar usuario por ID: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("✅ Usuario encontrado: {}", user.username),
            None => debug!("⚠️ Usuario no encontrado con ID: {}", id),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        debug!("🔍 Buscando usuario por email: {}", email);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::email.eq(email.to_lowercase()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("❌ Error al buscar usuario por email: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("✅ Usuario encontrado: {}", user.username),
            None => debug!("⚠️ Usuario no encontrado con email: {}", email),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApplicationError> {
        debug!("🔍 Buscando usuario por username: {}", username);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::username.eq(username))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("❌ Error al buscar usuario por username: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("✅ Usuario encontrado: {}", user.username),
            None => debug!("⚠️ Usuario no encontrado con username: {}", username),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_email_or_username(&self, identifier: &str) -> Result<Option<User>, ApplicationError> {
        debug!("🔍 Buscando usuario por email o username: {}", identifier);
        let mut conn = self.pool.get_connection().await?;
        let identifier_lower = identifier.to_lowercase();
        
        let result = users::table
            .filter(
                users::email.eq(&identifier_lower)
                    .or(users::username.eq(identifier))
            )
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("❌ Error al buscar usuario por email/username: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("✅ Usuario encontrado: {} (id: {})", user.username, user.id),
            None => debug!("⚠️ Usuario no encontrado con identifier: {}", identifier),
        }
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, user: &User) -> Result<User, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(users::table.filter(users::id.eq(user.id)))
            .set((
                users::username.eq(&user.username),
                users::email.eq(&user.email),
                users::password_hash.eq(&user.password_hash),
                users::role.eq(user.role.to_string()),
                users::id_entidad.eq(&user.id_entidad),
                users::nombre_entidad.eq(&user.nombre_entidad),
                users::status.eq(user.status.to_string()),
                users::last_login.eq(user.last_login),
                users::updated_by.eq(user.updated_by),
            ))
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::status.eq("inactivo"))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(())
    }
    
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = users::table
            .filter(users::email.eq(email.to_lowercase()))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    async fn exists_by_username(&self, username: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count: i64 = users::table
            .filter(users::username.eq(username))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    async fn list_active(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<User>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = users::table
            .filter(users::status.eq("activo"))
            .order(users::created_at.desc())
            .into_boxed();
        
        if let Some(l) = limit {
            query = query.limit(l);
        }
        
        if let Some(o) = offset {
            query = query.offset(o);
        }
        
        let results = query
            .load::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count_active(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let count = users::table
            .filter(users::status.eq("activo"))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count)
    }
    
    #[instrument(skip(self))]
    async fn list_users_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<crate::application::dtos::UserListItemDto>, i64), ApplicationError> {
        use crate::infrastructure::persistence::schema::personas;
        
        debug!("📋 Listando usuarios con detalles (limit: {}, offset: {})", limit, offset);
        let mut conn = self.pool.get_connection().await?;
        
        let total: i64 = users::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results: Vec<(UserModel, Option<(String, String)>)> = users::table
            .left_join(personas::table.on(users::id_persona.eq(personas::id.nullable())))
            .select((
                UserModel::as_select(),
                (personas::nombre, personas::apellidos).nullable(),
            ))
            .order(users::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<crate::application::dtos::UserListItemDto> = results
            .into_iter()
            .map(|(user, persona_data)| {
                let nombre_completo = persona_data.map(|(nombre, apellidos)| {
                    format!("{} {}", nombre, apellidos)
                });
                
                crate::application::dtos::UserListItemDto {
                    id: user.id,
                    nombre_completo,
                    username: user.username,
                    email: user.email,
                    role: user.role,
                    nombre_entidad: user.nombre_entidad,
                    status: user.status,
                    created_at: user.created_at,
                    last_login: user.last_login,
                }
            })
            .collect();
        
        info!("✅ Listados {} usuarios de {} total", items.len(), total);
        Ok((items, total))
    }
}
