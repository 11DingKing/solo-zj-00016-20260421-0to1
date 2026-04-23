use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::COOKIE, request::Parts, Request},
    middleware::Next,
    response::Response,
};

use crate::error::AppError;
use crate::utils::generate_user_cookie;

pub struct UserCookie(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for UserCookie
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(cookie) = parts.extensions.get::<UserCookie>() {
            return Ok(UserCookie(cookie.0.clone()));
        }

        let cookie_value = parts
            .headers
            .get(COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';')
                    .map(|s| s.trim())
                    .find(|s| s.starts_with("user_id="))
                    .map(|s| s.trim_start_matches("user_id=").to_string())
            });

        match cookie_value {
            Some(cookie) => Ok(UserCookie(cookie)),
            None => Err(AppError::NotFound),
        }
    }
}

pub async fn set_user_cookie<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let cookie_value = req
        .headers()
        .get(COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("user_id="))
                .map(|s| s.trim_start_matches("user_id=").to_string())
        });

    let (parts, body) = req.into_parts();
    
    let user_id = cookie_value.unwrap_or_else(generate_user_cookie);
    
    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(UserCookie(user_id.clone()));
    
    let mut response = next.run(req).await;
    
    let set_cookie = format!(
        "user_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000",
        user_id
    );
    
    response
        .headers_mut()
        .insert(
            axum::http::header::SET_COOKIE,
            set_cookie.parse().unwrap(),
        );
    
    response
}
