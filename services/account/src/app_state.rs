use crate::repositories::{
    account_repository::AccountRepository, audit_repository::AuditRepository,
};
use crate::services::account_service::AccountService;

#[derive(Clone)]
pub struct AppState {
    pub account_service: AccountService,
}

impl AppState {
    pub fn new() -> Self {
        let repository = AccountRepository::new();
        let audit_repository = AuditRepository::new();
        let account_service = AccountService::new(repository, audit_repository);
        Self { account_service }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
