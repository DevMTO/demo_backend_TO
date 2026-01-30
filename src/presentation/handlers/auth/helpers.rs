//! Helper functions para Auth handlers

use tower_cookies::Cookie;
use time::Duration;

use crate::infrastructure::container::DependencyContainer;

/// Crear cookie de sesión
pub fn create_session_cookie<'a>(
    token: &str,
    max_age_seconds: i64,
    container: &DependencyContainer,
) -> Cookie<'a> {
    let mut cookie = Cookie::new(
        container.cookie_name.clone(),
        token.to_string(),
    );
    cookie.set_http_only(container.cookie_http_only);
    cookie.set_secure(container.cookie_secure);
    cookie.set_path(container.cookie_path.clone());
    cookie.set_max_age(Duration::seconds(max_age_seconds));
    
    if !container.cookie_domain.is_empty() {
        cookie.set_domain(container.cookie_domain.clone());
    }
    
    cookie
}

/// Crear cookie de remoción de sesión (para logout)
pub fn remove_session_cookie(container: &DependencyContainer) -> Cookie<'static> {
    let mut cookie = Cookie::new(
        container.cookie_name.clone(),
        "",
    );
    cookie.set_http_only(container.cookie_http_only);
    cookie.set_secure(container.cookie_secure);
    cookie.set_path(container.cookie_path.clone());
    cookie.set_max_age(Duration::seconds(0));
    
    if !container.cookie_domain.is_empty() {
        cookie.set_domain(container.cookie_domain.clone());
    }
    
    cookie
}
