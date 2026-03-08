use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Cache: Clone + Send + Sync {
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: u64) -> bool;
    async fn delete(&self, key: &str) -> bool;
    async fn is_enabled(&self) -> bool;
}

#[derive(Clone)]
pub enum CacheClient {
    Redis(RedisCache),
    NoOp(NoOpCache),
}

#[async_trait]
impl Cache for CacheClient {
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        match self {
            CacheClient::Redis(cache) => cache.get(key).await,
            CacheClient::NoOp(cache) => cache.get(key).await,
        }
    }

    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: u64) -> bool {
        match self {
            CacheClient::Redis(cache) => cache.set(key, value, ttl).await,
            CacheClient::NoOp(cache) => cache.set(key, value, ttl).await,
        }
    }

    async fn delete(&self, key: &str) -> bool {
        match self {
            CacheClient::Redis(cache) => cache.delete(key).await,
            CacheClient::NoOp(cache) => cache.delete(key).await,
        }
    }

    async fn is_enabled(&self) -> bool {
        match self {
            CacheClient::Redis(cache) => cache.is_enabled().await,
            CacheClient::NoOp(cache) => cache.is_enabled().await,
        }
    }
}

#[derive(Clone)]
pub struct RedisCache {
    client: Option<redis::aio::ConnectionManager>,
    ttl: u64,
    enabled: bool,
}

impl RedisCache {
    pub async fn new(redis_url: &str, ttl: u64, enabled: bool) -> Self {
        if !enabled {
            log::info!("Cache disabled via configuration");
            return Self {
                client: None,
                ttl,
                enabled: false,
            };
        }

        match redis::Client::open(redis_url) {
            Ok(client) => match client.get_connection_manager().await {
                Ok(manager) => {
                    log::info!("Redis cache enabled and connected to: {}", redis_url);
                    Self {
                        client: Some(manager),
                        ttl,
                        enabled: true,
                    }
                }
                Err(e) => {
                    log::warn!(
                        "Failed to connect to Redis at {}: {}. Falling back to database access.",
                        redis_url,
                        e
                    );
                    Self {
                        client: None,
                        ttl,
                        enabled: false,
                    }
                }
            },
            Err(e) => {
                log::warn!(
                    "Invalid Redis URL {}: {}. Falling back to database access.",
                    redis_url,
                    e
                );
                Self {
                    client: None,
                    ttl,
                    enabled: false,
                }
            }
        }
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        match &self.client {
            None => {
                log::debug!("Cache miss (Redis unavailable): {}", key);
                None
            }
            Some(client) => {
                match redis::cmd("GET")
                    .arg(key)
                    .query_async::<_, Option<String>>(client)
                    .await
                {
                    Ok(Some(data)) => match serde_json::from_str::<T>(&data) {
                        Ok(value) => {
                            log::debug!("Cache hit: {}", key);
                            Some(value)
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize cache value for {}: {}", key, e);
                            None
                        }
                    },
                    Ok(None) => {
                        log::debug!("Cache miss: {}", key);
                        None
                    }
                    Err(e) => {
                        log::warn!("Cache get error for {}: {}", key, e);
                        None
                    }
                }
            }
        }
    }

    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: u64) -> bool {
        match &self.client {
            None => {
                log::debug!("Cache set skipped (Redis unavailable): {}", key);
                false
            }
            Some(client) => match serde_json::to_string(value) {
                Ok(serialized) => {
                    match redis::cmd("SET")
                        .arg(key)
                        .arg(&serialized)
                        .arg("EX")
                        .arg(ttl)
                        .query_async::<_, ()>(client)
                        .await
                    {
                        Ok(_) => {
                            log::debug!("Cache set with TTL {} seconds: {}", ttl, key);
                            true
                        }
                        Err(e) => {
                            log::warn!("Cache set error for {}: {}", key, e);
                            false
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to serialize value for cache key {}: {}", key, e);
                    false
                }
            },
        }
    }

    async fn delete(&self, key: &str) -> bool {
        match &self.client {
            None => {
                log::debug!("Cache delete skipped (Redis unavailable): {}", key);
                false
            }
            Some(client) => {
                match redis::cmd("DEL")
                    .arg(key)
                    .query_async::<_, u32>(client)
                    .await
                {
                    Ok(deleted) => {
                        if deleted > 0 {
                            log::debug!("Cache invalidated: {}", key);
                        }
                        true
                    }
                    Err(e) => {
                        log::warn!("Cache delete error for {}: {}", key, e);
                        false
                    }
                }
            }
        }
    }

    async fn is_enabled(&self) -> bool {
        self.enabled && self.client.is_some()
    }
}

#[derive(Clone)]
pub struct NoOpCache;

#[async_trait]
impl Cache for NoOpCache {
    async fn get<T: for<'de> Deserialize<'de>>(&self, _key: &str) -> Option<T> {
        None
    }

    async fn set<T: Serialize>(&self, _key: &str, _value: &T, _ttl: u64) -> bool {
        false
    }

    async fn delete(&self, _key: &str) -> bool {
        false
    }

    async fn is_enabled(&self) -> bool {
        false
    }
}
