use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub rate_limit_per_minute: u64,
    pub cache_ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://shortlink:shortlink123@localhost:5432/shortlink".to_string());
        
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let rate_limit_per_minute = env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        
        let cache_ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        Self {
            database_url,
            redis_url,
            rate_limit_per_minute,
            cache_ttl_seconds,
        }
    }
}
