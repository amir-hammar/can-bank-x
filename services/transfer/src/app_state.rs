use crate::{
    cache::{CacheClient, CacheConfig},
    config::env::AppConfig,
    repositories::{audit_repository::AuditRepository, transfer_repository::TransferRepository},
    services::transfer_service::TransferService,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub transfer_service: TransferService,
}

impl AppState {
    pub async fn new(config: AppConfig, pool: PgPool) -> Self {
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

        let transfer_repository = TransferRepository::new(pool.clone());
        let audit_repository = AuditRepository::new(pool);

        Self {
            transfer_service: TransferService::new(config, transfer_repository, audit_repository, cache),
        }
    }
}
