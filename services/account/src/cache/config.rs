use std::env;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub redis_url: String,
}

impl CacheConfig {
    pub fn from_env() -> Self {
        let enabled = env::var("CACHE_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or(300);

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        Self {
            enabled,
            ttl_seconds,
            redis_url,
        }
    }
}
