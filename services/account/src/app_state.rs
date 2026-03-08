use sqlx::PgPool;

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
        let repository = AccountRepository::new(pool.clone());
        let audit_repository = AuditRepository::new(pool);
        let account_service = AccountService::new(repository, audit_repository);
        Self { account_service }
    }
}
