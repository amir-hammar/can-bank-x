use sqlx::PgPool;

use crate::cache::{CacheClient, CacheConfig};
use crate::repositories::{
    account_repository::AccountRepository, audit_repository::AuditRepository,
};
use crate::services::account_service::AccountService;

#[derive(Clone)]
pub struct AppState {
    pub account_service: AccountService,
}

impl AppState {
    pub async fn new(pool: PgPool) -> Self {
        let cache_config = CacheConfig::from_env();
        let cache = if cache_config.enabled {
            CacheClient::Redis(
                crate::cache::client::RedisCache::new(
                    &cache_config.redis_url,
                    cache_config.ttl_seconds,
                    true,
                )
                .await,
            )
        } else {
            CacheClient::NoOp(crate::cache::client::NoOpCache)
        };

        let repository = AccountRepository::new(pool.clone());
        let audit_repository = AuditRepository::new(pool);
        let account_service = AccountService::new(repository, audit_repository, cache);
        Self { account_service }
    }
}
