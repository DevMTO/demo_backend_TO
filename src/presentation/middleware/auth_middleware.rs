
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApplicationError> {
    let cookie_name = &state.container.cookie_name;
    let token = extract_token_from_request(&request, cookie_name);
    
    match token {
        Some(token) => {
            let _verification = state.container.verify_session_use_case
                .execute(&token)
                .await?;
            
            Ok(next.run(request).await)
        }
        None => {
            Err(ApplicationError::SessionRequired)
        }
    }
}

fn extract_token_from_request(request: &Request, cookie_name: &str) -> Option<String> {
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    if let Some(cookie_header) = request.headers().get("Cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie in cookies_str.split(';') {
                let cookie = cookie.trim();
                if let Some((name, value)) = cookie.split_once('=') {
                    if name == cookie_name {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    
    None
}
