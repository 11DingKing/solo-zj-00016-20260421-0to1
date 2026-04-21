use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequestParts, Request},
    http::header::COOKIE,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::AppState;
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: Arc<AppState>,
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

pub struct ClientIp(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ip = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                parts.headers.get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "127.0.0.1".to_string());

        Ok(ClientIp(ip))
    }
}

pub async fn rate_limit<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: Arc<AppState>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let path = req.uri().path();
    
    if path != "/api/links" || req.method() != axum::http::Method::POST {
        return next.run(req).await;
    }

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.headers().get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| addr.ip().to_string());

    let count = match state.cache.get_rate_limit_count(&ip).await {
        Ok(c) => c,
        Err(_) => {
            return next.run(req).await;
        }
    };

    if count >= state.config.rate_limit_per_minute {
        return AppError::RateLimitExceeded.into_response();
    }

    let _ = state.cache.increment_rate_limit(&ip).await;

    next.run(req).await
}
