use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, warn, info, instrument};

use crate::application::ports::{UserRepositoryPort, UserListScope};
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
        debug!("Creando usuario: {}", user.username);
        let mut conn = self.pool.get_connection().await?;
        let new_user: NewUserModel = user.into();
        
        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al crear usuario: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("Usuario creado: {} (id: {})", result.username, result.id);
        Ok(result.into())
    }
    
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, ApplicationError> {
        debug!("Buscando usuario por ID: {}", id);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::id.eq(id))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("Error al buscar usuario por ID: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("Usuario encontrado: {}", user.username),
            None => debug!("Usuario no encontrado con ID: {}", id),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        debug!("Buscando usuario por email: {}", email);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::email.eq(email.to_lowercase()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("Error al buscar usuario por email: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("Usuario encontrado: {}", user.username),
            None => debug!("Usuario no encontrado con email: {}", email),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApplicationError> {
        debug!("Buscando usuario por username: {}", username);
        let mut conn = self.pool.get_connection().await?;
        
        let result = users::table
            .filter(users::username.eq(username))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                warn!("Error al buscar usuario por username: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("Usuario encontrado: {}", user.username),
            None => debug!("Usuario no encontrado con username: {}", username),
        }
        
        Ok(result.map(Into::into))
    }
    
    #[instrument(skip(self))]
    async fn find_by_email_or_username(&self, identifier: &str) -> Result<Option<User>, ApplicationError> {
        debug!("Buscando usuario por email o username: {}", identifier);
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
                warn!("Error al buscar usuario por email/username: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        match &result {
            Some(user) => debug!("Usuario encontrado: {} (id: {})", user.username, user.id),
            None => debug!("Usuario no encontrado con identifier: {}", identifier),
        }
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, user: &User) -> Result<User, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let result = diesel::update(users::table.filter(users::id.eq(user.id)))
            .set((
                users::id_persona.eq(&user.id_persona),
                users::username.eq(&user.username),
                users::email.eq(&user.email),
                users::password_hash.eq(&user.password_hash),
                users::role.eq(user.role.to_string()),
                users::id_entidad.eq(&user.id_entidad),
                users::is_active.eq(user.is_active),
                users::is_demo.eq(user.is_demo),
                users::demo_expires_at.eq(&user.demo_expires_at),
                users::last_login.eq(user.last_login),
                users::updated_by.eq(user.updated_by),
                users::turno.eq(&user.turno),
            ))
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(())
    }
    
    /// Eliminación permanente de la base de datos (hard delete)
    async fn hard_delete(&self, id: i32) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::delete(users::table.filter(users::id.eq(id)))
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
            .filter(users::is_active.eq(true))
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
            .filter(users::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(count)
    }
    
    #[instrument(skip(self))]
    async fn list_users_with_details(&self, limit: i64, offset: i64, is_demo: Option<bool>, scope: &UserListScope) -> Result<(Vec<crate::application::dtos::UserListItemDto>, i64), ApplicationError> {
        use crate::infrastructure::persistence::schema::{personas, hoteles};
        use diesel::dsl::count_star;

        debug!("Listando usuarios con detalles (limit: {}, offset: {}, is_demo: {:?}, scope: {:?})", limit, offset, is_demo, scope);
        let mut conn = self.pool.get_connection().await?;

        // Pre-fetch hotel IDs for cadena scope (requires an extra query before the main one)
        let hotel_ids: Vec<i32> = if let UserListScope::HotelCadenaScope { id_cadena } = scope {
            hoteles::table
                .filter(hoteles::id_cadena.eq(id_cadena))
                .select(hoteles::id)
                .load::<i32>(&mut conn)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?
        } else {
            vec![]
        };

        // Compute (roles, entity_ids) to filter by, if any scope restriction applies
        let scope_filter: Option<(Vec<String>, Vec<i32>)> = match scope {
            UserListScope::All => None,
            UserListScope::Empty => Some((vec![], vec![0])),  // Empty result - impossible match
            UserListScope::AgenciaScope { id_entidad } => Some((
                vec!["agencias_gerente".into(), "agencias".into(), "agencias_contador".into()],
                vec![*id_entidad],
            )),
            UserListScope::HotelCadenaScope { id_cadena } => {
                let mut entity_ids = vec![*id_cadena];
                entity_ids.extend_from_slice(&hotel_ids);
                Some((
                    vec!["hoteles_gerente_cadena".into(), "hoteles_gerente".into(), "hoteles".into()],
                    entity_ids,
                ))
            },
            UserListScope::HotelScope { id_hotel } => Some((
                vec!["hoteles_gerente".into(), "hoteles".into()],
                vec![*id_hotel],
            )),
        };

        // Count query using SELECT COUNT(*) with a boxed query so we can apply filters dynamically
        let mut count_q = users::table
            .select(count_star())
            .into_boxed::<diesel::pg::Pg>();
        if let Some(demo) = is_demo {
            count_q = count_q.filter(users::is_demo.eq(demo));
        }
        if let Some((ref roles, ref entity_ids)) = scope_filter {
            count_q = count_q
                .filter(users::role.eq_any(roles))
                .filter(users::id_entidad.eq_any(entity_ids));
        }
        let total: i64 = count_q
            .first(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Data query with persona join, also boxed for dynamic filters
        let mut data_q = users::table
            .left_join(personas::table.on(users::id_persona.eq(personas::id.nullable())))
            .select((UserModel::as_select(), (personas::nombre, personas::apellidos).nullable()))
            .order(users::created_at.desc())
            .into_boxed::<diesel::pg::Pg>();
        if let Some(demo) = is_demo {
            data_q = data_q.filter(users::is_demo.eq(demo));
        }
        if let Some((ref roles, ref entity_ids)) = scope_filter {
            data_q = data_q
                .filter(users::role.eq_any(roles))
                .filter(users::id_entidad.eq_any(entity_ids));
        }
        let results: Vec<(UserModel, Option<(String, String)>)> = data_q
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Map results to DTOs
        let items: Vec<crate::application::dtos::UserListItemDto> = results
            .into_iter()
            .map(|(user, persona_data)| {
                let nombre_completo = persona_data.map(|(nombre, apellidos)| format!("{} {}", nombre, apellidos));
                crate::application::dtos::UserListItemDto {
                    id: user.id,
                    nombre_completo,
                    username: user.username,
                    email: user.email,
                    role: user.role,
                    is_active: user.is_active,
                    created_at: user.created_at,
                    last_login: user.last_login,
                    id_persona: user.id_persona,
                    id_entidad: user.id_entidad,
                    turno: user.turno,
                    is_demo: user.is_demo,
                    demo_expires_at: user.demo_expires_at,
                }
            })
            .collect();

        info!("Listados {} usuarios de {} total", items.len(), total);
        Ok((items, total))
    }
    
    /// Encuentra usuarios por rol e id_entidad (para notificaciones a proveedores)
    #[instrument(skip(self))]
    async fn find_by_role_and_entity(&self, role: &str, entity_id: i32) -> Result<Vec<User>, ApplicationError> {
        debug!("Buscando usuarios con rol '{}' y entidad {}", role, entity_id);
        let mut conn = self.pool.get_connection().await?;
        
        let results = users::table
            .filter(users::role.eq(role))
            .filter(users::id_entidad.eq(entity_id))
            .filter(users::is_active.eq(true))
            .load::<UserModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al buscar usuarios por rol y entidad: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("Encontrados {} usuarios con rol '{}' y entidad {}", results.len(), role, entity_id);
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    /// Encuentra usuarios por id_persona
    #[instrument(skip(self))]
    async fn find_by_persona_id(&self, persona_id: i32) -> Result<Vec<User>, ApplicationError> {
        debug!("Buscando usuarios con persona_id {}", persona_id);
        let mut conn = self.pool.get_connection().await?;
        
        let results = users::table
            .filter(users::id_persona.eq(persona_id))
            .filter(users::is_active.eq(true))
            .load::<UserModel>(&mut conn)
            .await
            .map_err(|e| {
                warn!("Error al buscar usuarios por persona_id: {}", e);
                ApplicationError::Repository(e.to_string())
            })?;
        
        info!("Encontrados {} usuarios con persona_id {}", results.len(), persona_id);
        Ok(results.into_iter().map(Into::into).collect())
    }
}
