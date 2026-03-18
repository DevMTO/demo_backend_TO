
use diesel_async::scoped_futures::ScopedBoxFuture;
use diesel_async::AsyncConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::pg::PgConnection;
use diesel::Connection;

use crate::config::AppConfig;
use crate::domain::errors::ApplicationError;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Clone)]
pub struct DatabasePool {
    pool: DbPool,
    database_url: String,
}

impl DatabasePool {
    /// Crear un nuevo pool de conexiones asíncronas
    pub async fn new(config: &AppConfig) -> Result<Self, ApplicationError> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);
        
        let pool = Pool::builder(manager)
            .max_size(config.database_max_connections as usize)
            .build()
            .map_err(|e| ApplicationError::Configuration(format!("Database pool error: {}", e)))?;
        
        // Verificar conexión
        let _conn = pool.get().await
            .map_err(|e| ApplicationError::Configuration(format!("Database connection test failed: {}", e)))?;
        
        Ok(Self { 
            pool,
            database_url: config.database_url.clone(),
        })
    }
    
    /// Obtener una conexión del pool
    pub async fn get_connection(&self) -> Result<deadpool::managed::Object<AsyncDieselConnectionManager<AsyncPgConnection>>, ApplicationError> {
        self.pool
            .get()
            .await
            .map_err(|e| ApplicationError::Repository(format!("Connection error: {}", e)))
    }
    
    /// Obtener el pool interno
    #[allow(dead_code)]
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
    
    /// Ejecutar la función diaria de automatización de estados
    pub async fn run_daily_automation(&self) -> Result<(), ApplicationError> {
        let mut conn = self.get_connection().await?;
        diesel::sql_query("SELECT automatizar_estados_por_fecha()")
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Daily automation error: {}", e)))?;
        Ok(())
    }

    /// Ejecutar migraciones (usa conexión síncrona temporal)
    pub async fn run_migrations(&self) -> Result<(), ApplicationError> {
        // Las migraciones de Diesel necesitan una conexión síncrona
        let database_url = self.database_url.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| ApplicationError::Configuration(format!("Migration connection error: {}", e)))?;
            
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|e| ApplicationError::Configuration(format!("Migration error: {}", e)))?;
            
            Ok::<(), ApplicationError>(())
        })
        .await
        .map_err(|e| ApplicationError::Configuration(format!("Migration task error: {}", e)))??;
        
        Ok(())
    }

    /// Ejecuta una función dentro de una transacción de base de datos.
    /// Si la función devuelve un error, se hace rollback automáticamente.
    pub async fn with_transaction<'a, R, E, F>(&self, f: F) -> Result<R, E>
    where
        F: for<'r> FnOnce(&'r mut deadpool::managed::Object<AsyncDieselConnectionManager<AsyncPgConnection>>) -> ScopedBoxFuture<'a, 'r, Result<R, E>> + Send + 'a,
        E: From<diesel::result::Error> + From<ApplicationError> + Send + 'a,
        R: Send + 'a,
    {
        let mut conn = self.get_connection().await
            .map_err(E::from)?;
        diesel_async::AsyncConnection::transaction(&mut conn, f).await
    }
}
