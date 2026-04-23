use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::AppState;
use crate::error::AppError;
use crate::middleware::UserCookie;
use crate::models::{
    CreateShortLinkRequest, ShortLinkResponse, StatsResponse, UserLinksResponse,
};
use crate::utils::{generate_short_code, validate_url};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub async fn create_short_link(
    State(state): State<Arc<AppState>>,
    user_cookie: UserCookie,
    headers: HeaderMap,
    Json(payload): Json<CreateShortLinkRequest>,
) -> Result<Json<ShortLinkResponse>, AppError> {
    if !validate_url(&payload.original_url) {
        return Err(AppError::InvalidUrl);
    }

    let mut short_code = generate_short_code(6);
    let mut attempts = 0;
    
    while attempts < 5 {
        let existing = state.db.get_short_link(&short_code).await?;
        if existing.is_none() {
            break;
        }
        short_code = generate_short_code(6);
        attempts += 1;
    }

    if attempts >= 5 {
        return Err(AppError::Internal);
    }

    let link = state
        .db
        .create_short_link(
            &short_code,
            &payload.original_url,
            &user_cookie.0,
            payload.expires_in_hours,
        )
        .await?;

    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8080");
    
    let scheme = if host.contains("localhost") { "http" } else { "https" };
    let short_url = format!("{}://{}/{}", scheme, host, short_code);

    Ok(Json(ShortLinkResponse {
        short_code: link.short_code,
        original_url: link.original_url,
        short_url,
        created_at: link.created_at,
        expires_at: link.expires_at,
        total_clicks: 0,
    }))
}

pub async fn get_user_links(
    State(state): State<Arc<AppState>>,
    user_cookie: UserCookie,
    headers: HeaderMap,
) -> Result<Json<UserLinksResponse>, AppError> {
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8080");
    
    let scheme = if host.contains("localhost") { "http" } else { "https" };
    let base_url = format!("{}://{}", scheme, host);

    let links = state.db.get_user_links(&user_cookie.0, &base_url).await?;

    Ok(Json(UserLinksResponse { links }))
}

pub async fn get_link_stats(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
) -> Result<Json<StatsResponse>, AppError> {
    let link = state
        .db
        .get_short_link(&short_code)
        .await?
        .ok_or(AppError::NotFound)?;

    let total_clicks = state.db.get_total_clicks(link.id).await?;
    let daily_clicks = state.db.get_daily_clicks(link.id, 7).await?;
    let top_referers = state.db.get_top_referers(link.id, 5).await?;

    Ok(Json(StatsResponse {
        total_clicks,
        daily_clicks,
        top_referers,
    }))
}

pub async fn redirect_short_link(
    State(state): State<Arc<AppState>>,
    Path(short_code): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Response, AppError> {
    let cached_url = state.cache.get_url(&short_code).await?;
    
    let link = if let Some(url) = cached_url {
        let db_link = state
            .db
            .get_short_link(&short_code)
            .await?
            .ok_or(AppError::NotFound)?;
        Some(db_link)
    } else {
        let db_link = state
            .db
            .get_short_link(&short_code)
            .await?;
        
        if let Some(ref link) = db_link {
            state
                .cache
                .set_url(&short_code, &link.original_url, state.config.cache_ttl_seconds)
                .await?;
        }
        
        db_link
    };

    let link = link.ok_or(AppError::NotFound)?;

    if let Some(expires_at) = link.expires_at {
        if Utc::now() > expires_at {
            return Err(AppError::Expired);
        }
    }

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers.get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| addr.ip().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let referer = headers
        .get("referer")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    state
        .db
        .record_visit(
            link.id,
            Some(&ip),
            user_agent.as_deref(),
            referer.as_deref(),
        )
        .await?;

    Ok(Redirect::found(&link.original_url).into_response())
}
