use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub success: bool,
    pub message: String,
}

impl MessageResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

impl PaginationParams {
    pub fn to_options(&self) -> crate::application::ports::PaginationOptions {
        let limit = self.page_size.min(10000); // Aumentado de 100 a 10000 para permitir cargar todos los registros
        let offset = (self.page - 1).max(0) * limit;
        crate::application::ports::PaginationOptions {
            limit: Some(limit),
            offset: Some(offset),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[allow(dead_code)]
pub fn paginated_response<T, R>(
    items: Vec<T>,
    total: i64,
    page: i64,
    page_size: i64,
) -> PaginatedResponse<R> 
where
    R: Serialize + From<T>,
{
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    PaginatedResponse {
        items: items.into_iter().map(Into::into).collect(),
        pagination: PaginationInfo {
            page,
            page_size,
            total,
            total_pages,
        },
    }
}

pub fn create_paginated_response<T: Serialize>(
    items: Vec<T>,
    total: i64,
    page: i64,
    page_size: i64,
) -> PaginatedResponse<T> {
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    PaginatedResponse {
        items,
        pagination: PaginationInfo {
            page,
            page_size,
            total,
            total_pages,
        },
    }
}

pub fn json_ok<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::OK, Json(ApiResponse::success(data)))
}

pub fn json_created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::success(data)))
}

pub fn json_message(message: impl Into<String>) -> impl IntoResponse {
    (StatusCode::OK, Json(MessageResponse::success(message)))
}

pub fn json_deleted() -> impl IntoResponse {
    json_message("Registro eliminado correctamente")
}
