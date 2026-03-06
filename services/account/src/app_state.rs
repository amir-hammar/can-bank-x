use crate::repositories::account_repository::AccountRepository;
use crate::services::account_service::AccountService;

#[derive(Clone)]
pub struct AppState {
    pub account_service: AccountService,
}

impl AppState {
    pub fn new() -> Self {
        let repository = AccountRepository::new();
        let account_service = AccountService::new(repository);
        Self { account_service }
    }
}
