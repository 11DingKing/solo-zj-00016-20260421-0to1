use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShortLink {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub user_cookie: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkVisit {
    pub id: i32,
    pub short_link_id: i32,
    pub visited_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShortLinkRequest {
    pub original_url: String,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLinkResponse {
    pub short_code: String,
    pub original_url: String,
    pub short_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub total_clicks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLinkListItem {
    pub short_code: String,
    pub original_url: String,
    pub short_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub total_clicks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_clicks: i64,
    pub daily_clicks: Vec<DailyClick>,
    pub top_referers: Vec<RefererStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyClick {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefererStat {
    pub referer: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLinksResponse {
    pub links: Vec<UserLinkListItem>,
}
