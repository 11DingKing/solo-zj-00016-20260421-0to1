use redis::Client;
use std::sync::Arc;

use crate::error::AppError;

#[derive(Clone)]
pub struct CachePool {
    client: Arc<Client>,
}

impl CachePool {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await?;
        
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn get_url(&self, short_code: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("url:{}", short_code);
        let result: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(result)
    }

    pub async fn set_url(
        &self,
        short_code: &str,
        original_url: &str,
        ttl_seconds: u64,
    ) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("url:{}", short_code);
        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_seconds)
            .arg(original_url)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn get_rate_limit_count(&self, ip: &str) -> Result<u64, AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ratelimit:{}", ip);
        let count: Option<u64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        Ok(count.unwrap_or(0))
    }

    pub async fn increment_rate_limit(&self, ip: &str) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ratelimit:{}", ip);
        
        let current: Option<u64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        if current.is_none() {
            redis::cmd("SETEX")
                .arg(&key)
                .arg(60u64)
                .arg(1u64)
                .query_async(&mut conn)
                .await?;
        } else {
            redis::cmd("INCR")
                .arg(&key)
                .query_async(&mut conn)
                .await?;
        }
        
        Ok(())
    }
}
