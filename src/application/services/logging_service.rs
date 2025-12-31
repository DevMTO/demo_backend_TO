use std::sync::Arc;
use tracing::{debug, error, instrument};

use crate::application::ports::{ActivityLogRepositoryPort, ActivityLogFilters, PaginationOptions};
use crate::application::dtos::{ActivityLogSummaryDto, ActionTypeSummary, StatusSummary};
use crate::domain::entities::{
    ActivityLogBuilder, ActionType, Action, EntityType, LogStatus, NewActivityLog,
};
use crate::domain::errors::ApplicationError;

#[allow(dead_code)]
pub struct LoggingService {
    repository: Arc<dyn ActivityLogRepositoryPort>,
}

#[allow(dead_code)]
impl LoggingService {
    pub fn new(repository: Arc<dyn ActivityLogRepositoryPort>) -> Self {
        Self { repository }
    }
    /// Log de login exitoso
    #[instrument(skip(self))]
    pub async fn log_login(
        &self,
        user_id: i32,
        username: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), ApplicationError> {
        let log = ActivityLogBuilder::new()
            .user(user_id, username)
            .action_type(ActionType::Auth)
            .action(Action::Login)
            .entity(EntityType::Session, None)
            .description(format!("Usuario {} inició sesión", username))
            .request_info(ip, user_agent)
            .status(LogStatus::Success)
            .build();

        self.create_log(log).await
    }

    /// Log de login fallido
    #[instrument(skip(self))]
    pub async fn log_login_failed(
        &self,
        identifier: &str,
        reason: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), ApplicationError> {
        let log = ActivityLogBuilder::new()
            .action_type(ActionType::Auth)
            .action(Action::LoginFailed)
            .entity(EntityType::Session, None)
            .description(format!("Intento de login fallido para: {}", identifier))
            .request_info(ip, user_agent)
            .error(reason)
            .build();

        self.create_log(log).await
    }

    /// Log de logout
    #[instrument(skip(self))]
    pub async fn log_logout(
        &self,
        user_id: i32,
        username: &str,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let log = ActivityLogBuilder::new()
            .user(user_id, username)
            .action_type(ActionType::Auth)
            .action(Action::Logout)
            .entity(EntityType::Session, None)
            .description(format!("Usuario {} cerró sesión", username))
            .request_info(ip, None)
            .status(LogStatus::Success)
            .build();

        self.create_log(log).await
    }

    /// Log de creación de entidad
    #[instrument(skip(self, new_values))]
    pub async fn log_create<T: serde::Serialize>(
        &self,
        user_id: Option<i32>,
        username: Option<String>,
        entity_type: EntityType,
        entity_id: i32,
        entity_name: &str,
        new_values: Option<&T>,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let mut builder = ActivityLogBuilder::new()
            .action_type(ActionType::Crud)
            .action(Action::Create)
            .entity(entity_type.clone(), Some(entity_id))
            .description(format!("Creado {} con ID {}", entity_type.as_str(), entity_id))
            .request_info(ip, None)
            .status(LogStatus::Success);

        if let Some(uid) = user_id {
            if let Some(uname) = username {
                builder = builder.user(uid, uname);
            }
        }

        if let Some(values) = new_values {
            if let Ok(json) = serde_json::to_value(values) {
                builder = builder.new_values(json);
            }
        }

        self.create_log(builder.build()).await
    }

    /// Log de actualización de entidad
    #[instrument(skip(self, old_values, new_values, changed_fields))]
    pub async fn log_update<T: serde::Serialize>(
        &self,
        user_id: Option<i32>,
        username: Option<String>,
        entity_type: EntityType,
        entity_id: i32,
        old_values: Option<&T>,
        new_values: Option<&T>,
        changed_fields: Option<Vec<String>>,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let mut builder = ActivityLogBuilder::new()
            .action_type(ActionType::Crud)
            .action(Action::Update)
            .entity(entity_type.clone(), Some(entity_id))
            .description(format!("Actualizado {} con ID {}", entity_type.as_str(), entity_id))
            .request_info(ip, None)
            .status(LogStatus::Success);

        if let Some(uid) = user_id {
            if let Some(uname) = username {
                builder = builder.user(uid, uname);
            }
        }

        if let Some(old) = old_values {
            if let Ok(json) = serde_json::to_value(old) {
                builder = builder.old_values(json);
            }
        }

        if let Some(new) = new_values {
            if let Ok(json) = serde_json::to_value(new) {
                builder = builder.new_values(json);
            }
        }

        if let Some(fields) = changed_fields {
            builder = builder.changed_fields(fields);
        }

        self.create_log(builder.build()).await
    }

    /// Log de eliminación de entidad
    #[instrument(skip(self, old_values))]
    pub async fn log_delete<T: serde::Serialize>(
        &self,
        user_id: Option<i32>,
        username: Option<String>,
        entity_type: EntityType,
        entity_id: i32,
        old_values: Option<&T>,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let mut builder = ActivityLogBuilder::new()
            .action_type(ActionType::Crud)
            .action(Action::Delete)
            .entity(entity_type.clone(), Some(entity_id))
            .description(format!("Eliminado {} con ID {}", entity_type.as_str(), entity_id))
            .request_info(ip, None)
            .status(LogStatus::Success);

        if let Some(uid) = user_id {
            if let Some(uname) = username {
                builder = builder.user(uid, uname);
            }
        }

        if let Some(old) = old_values {
            if let Ok(json) = serde_json::to_value(old) {
                builder = builder.old_values(json);
            }
        }

        self.create_log(builder.build()).await
    }

    /// Log de error genérico
    #[instrument(skip(self))]
    pub async fn log_error(
        &self,
        user_id: Option<i32>,
        username: Option<String>,
        action: Action,
        entity_type: EntityType,
        entity_id: Option<i32>,
        error_message: &str,
        ip: Option<String>,
    ) -> Result<(), ApplicationError> {
        let mut builder = ActivityLogBuilder::new()
            .action_type(ActionType::System)
            .action(action)
            .entity(entity_type, entity_id)
            .request_info(ip, None)
            .error(error_message);

        if let Some(uid) = user_id {
            if let Some(uname) = username {
                builder = builder.user(uid, uname);
            }
        }

        self.create_log(builder.build()).await
    }

    // ===== Métodos de bajo nivel =====

    /// Crear log directamente
    async fn create_log(&self, log: NewActivityLog) -> Result<(), ApplicationError> {
        debug!("📝 Creando log: {} - {}", log.action_type, log.action);
        
        // Ejecutar en background para no bloquear la operación principal
        match self.repository.create(log).await {
            Ok(_) => {
                debug!("✅ Log creado exitosamente");
                Ok(())
            }
            Err(e) => {
                // Los errores de logging no deberían afectar la operación principal
                error!("❌ Error al crear log (no crítico): {}", e);
                Ok(()) // Retornamos Ok para no interrumpir el flujo
            }
        }
    }

    /// Listar logs con filtros
    #[instrument(skip(self, filters))]
    pub async fn list_logs(
        &self,
        filters: ActivityLogFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::domain::entities::ActivityLog>, ApplicationError> {
        let pagination = PaginationOptions {
            limit: Some(limit),
            offset: Some(offset),
        };

        let result = self.repository.find_all(filters, pagination).await?;
        Ok(result.data)
    }

    /// Obtener resumen estadístico
    #[instrument(skip(self))]
    pub async fn get_summary(&self) -> Result<ActivityLogSummaryDto, ApplicationError> {
        let by_action_type = self.repository.count_by_action_type().await?;
        let by_status = self.repository.count_by_status().await?;
        let recent_errors = self.repository.find_recent_errors(10).await?;

        let total_logs: i64 = by_action_type.iter().map(|c| c.count).sum();

        Ok(ActivityLogSummaryDto {
            total_logs,
            by_action_type: by_action_type
                .into_iter()
                .map(|c| ActionTypeSummary {
                    action_type: c.key,
                    count: c.count,
                })
                .collect(),
            by_status: by_status
                .into_iter()
                .map(|c| StatusSummary {
                    status: c.key,
                    count: c.count,
                })
                .collect(),
            recent_errors: recent_errors.into_iter().map(Into::into).collect(),
        })
    }
}
