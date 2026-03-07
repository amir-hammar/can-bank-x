use crate::{
    config::env::AppConfig,
    repositories::{audit_repository::AuditRepository, transfer_repository::TransferRepository},
    services::transfer_service::TransferService,
};

#[derive(Clone)]
pub struct AppState {
    pub transfer_service: TransferService,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let transfer_repository = TransferRepository::new();
        let audit_repository = AuditRepository::new();

        Self {
            transfer_service: TransferService::new(config, transfer_repository, audit_repository),
        }
    }
}
