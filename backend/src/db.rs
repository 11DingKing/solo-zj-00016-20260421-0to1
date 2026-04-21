use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::{DailyClick, RefererStat, ShortLink, UserLinkListItem};

#[derive(Clone)]
pub struct DbPool {
    pool: Arc<PgPool>,
}

impl DbPool {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool: Arc::new(pool) })
    }

    pub async fn create_short_link(
        &self,
        short_code: &str,
        original_url: &str,
        user_cookie: &str,
        expires_in_hours: Option<i64>,
    ) -> Result<ShortLink, AppError> {
        let expires_at = expires_in_hours.map(|hours| {
            Utc::now() + Duration::hours(hours)
        });

        let link: ShortLink = sqlx::query_as(
            r#"
            INSERT INTO short_links (short_code, original_url, user_cookie, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, short_code, original_url, user_cookie, expires_at, created_at, is_active
            "#
        )
        .bind(short_code)
        .bind(original_url)
        .bind(user_cookie)
        .bind(expires_at)
        .fetch_one(&*self.pool)
        .await?;

        Ok(link)
    }

    pub async fn get_short_link(&self, short_code: &str) -> Result<Option<ShortLink>, AppError> {
        let link: Option<ShortLink> = sqlx::query_as(
            r#"
            SELECT id, short_code, original_url, user_cookie, expires_at, created_at, is_active
            FROM short_links
            WHERE short_code = $1
            "#
        )
        .bind(short_code)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(link)
    }

    pub async fn get_user_links(&self, user_cookie: &str, base_url: &str) -> Result<Vec<UserLinkListItem>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                sl.short_code,
                sl.original_url,
                sl.created_at,
                sl.expires_at,
                COUNT(lv.id) as total_clicks
            FROM short_links sl
            LEFT JOIN link_visits lv ON sl.id = lv.short_link_id
            WHERE sl.user_cookie = $1
            GROUP BY sl.id, sl.short_code, sl.original_url, sl.created_at, sl.expires_at
            ORDER BY sl.created_at DESC
            "#
        )
        .bind(user_cookie)
        .fetch_all(&*self.pool)
        .await?;

        let links: Vec<UserLinkListItem> = rows
            .into_iter()
            .map(|row| {
                let short_code: String = row.get("short_code");
                UserLinkListItem {
                    short_code: short_code.clone(),
                    original_url: row.get("original_url"),
                    short_url: format!("{}/{}", base_url, short_code),
                    created_at: row.get("created_at"),
                    expires_at: row.get("expires_at"),
                    total_clicks: row.get::<i64, _>("total_clicks"),
                }
            })
            .collect();

        Ok(links)
    }

    pub async fn record_visit(
        &self,
        short_link_id: i32,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        referer: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO link_visits (short_link_id, ip_address, user_agent, referer)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(short_link_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(referer)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_total_clicks(&self, short_link_id: i32) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM link_visits WHERE short_link_id = $1
            "#
        )
        .bind(short_link_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(count.0)
    }

    pub async fn get_daily_clicks(&self, short_link_id: i32, days: i64) -> Result<Vec<DailyClick>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                DATE(visited_at AT TIME ZONE 'UTC') as date,
                COUNT(*) as count
            FROM link_visits
            WHERE short_link_id = $1 
                AND visited_at >= NOW() - INTERVAL '1 day' * $2
            GROUP BY DATE(visited_at AT TIME ZONE 'UTC')
            ORDER BY date DESC
            "#
        )
        .bind(short_link_id)
        .bind(days)
        .fetch_all(&*self.pool)
        .await?;

        let daily_clicks: Vec<DailyClick> = rows
            .into_iter()
            .map(|row| DailyClick {
                date: row.get::<chrono::NaiveDate, _>("date").to_string(),
                count: row.get("count"),
            })
            .collect();

        Ok(daily_clicks)
    }

    pub async fn get_top_referers(&self, short_link_id: i32, limit: i64) -> Result<Vec<RefererStat>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                COALESCE(referer, 'Direct') as referer,
                COUNT(*) as count
            FROM link_visits
            WHERE short_link_id = $1
            GROUP BY COALESCE(referer, 'Direct')
            ORDER BY count DESC
            LIMIT $2
            "#
        )
        .bind(short_link_id)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        let referers: Vec<RefererStat> = rows
            .into_iter()
            .map(|row| RefererStat {
                referer: row.get("referer"),
                count: row.get("count"),
            })
            .collect();

        Ok(referers)
    }
}
