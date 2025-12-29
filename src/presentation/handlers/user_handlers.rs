use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::{json_ok, create_paginated_response};

#[derive(Debug, Deserialize)]
pub struct ListUsersParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

#[instrument(skip(state))]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("📋 Listando usuarios (page: {}, size: {})", params.page, params.page_size);
    
    let page_size = params.page_size.min(100).max(1);
    let offset = (params.page - 1).max(0) * page_size;
    
    let (users, total) = state.container.user_repository
        .list_users_with_details(page_size, offset)
        .await?;
    
    let response = create_paginated_response(users, total, params.page, page_size);
    
    Ok(json_ok(response))
}
