use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::env::AppConfig,
    models::{
        dto::{
            AccountBalanceResponse, CreateTransferRequest, CreateTransferResponse,
            ListTransfersQuery, TransferDetailResponse, TransferSummaryResponse,
        },
        transfer::Transfer,
    },
    repositories::{
        audit_repository::AuditRepository,
        transfer_repository::{CreateTransferInput, TransferRepository},
    },
    utils::{errors::AppError, validators},
};

#[derive(Clone)]
pub struct TransferService {
    account_service_base_url: String,
    user_service_base_url: String,
    http_client: Client,
    transfer_repository: TransferRepository,
    audit_repository: AuditRepository,
}

#[derive(Debug, Serialize)]
struct AccountApplyTransferRequest {
    from_account_id: String,
    to_account_id: String,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct AccountApplyTransferResponse {
    currency: String,
}

#[derive(Debug, Deserialize)]
struct CustomerResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct DefaultAccountResponse {
    account_id: String,
}

impl TransferService {
    const AML_BLOCK_THRESHOLD: f64 = 5000.0;

    pub fn new(
        config: AppConfig,
        transfer_repository: TransferRepository,
        audit_repository: AuditRepository,
    ) -> Self {
        Self {
            account_service_base_url: config.account_service_base_url,
            user_service_base_url: config.user_service_base_url,
            http_client: Client::new(),
            transfer_repository,
            audit_repository,
        }
    }

    pub async fn create_transfer(
        &self,
        payload: CreateTransferRequest,
        trace_id: Option<String>,
    ) -> Result<CreateTransferResponse, AppError> {
        validators::validate_create_transfer(&payload)?;

        let to_account_id = self
            .resolve_beneficiary_account(&payload.beneficiary_username)
            .await?;

        if payload.amount >= Self::AML_BLOCK_THRESHOLD {
            self.audit_repository
                .append(
                    "SYSTEM",
                    "aml-engine",
                    "TRANSFER_BLOCKED_AML",
                    "TRANSFER_ATTEMPT",
                    &payload.idempotency_key,
                    trace_id,
                )
                .await;

            return Err(AppError::bad_request(
                "AML_BLOCKED",
                format!(
                    "transfer blocked by AML policy for amount >= {} CAD",
                    Self::AML_BLOCK_THRESHOLD
                ),
            ));
        }

        if let Some(existing) = self
            .transfer_repository
            .find_by_idempotency_key(&payload.customer_id, &payload.idempotency_key)
            .await
        {
            return Ok(self.to_create_response(existing));
        }

        let from_balance = self
            .fetch_account_balance(&payload.from_account_id)
            .await
            .map_err(|_| {
                AppError::failed_dependency(
                    "ACCOUNT_SERVICE_UNAVAILABLE",
                    "unable to validate source account balance",
                )
            })?;

        self.fetch_account_balance(&to_account_id)
            .await
            .map_err(|_| {
                AppError::failed_dependency(
                    "ACCOUNT_SERVICE_UNAVAILABLE",
                    "unable to validate destination account",
                )
            })?;

        let apply_result = self
            .apply_account_transfer(&payload.from_account_id, &to_account_id, payload.amount)
            .await
            .map_err(|_| {
                AppError::failed_dependency(
                    "ACCOUNT_TRANSFER_APPLY_FAILED",
                    "unable to apply account balance updates",
                )
            })?;

        if from_balance.available_balance < payload.amount {
            self.audit_repository
                .append(
                    "SYSTEM",
                    "transfer-service",
                    "TRANSFER_BLOCKED_INSUFFICIENT_FUNDS",
                    "TRANSFER_ATTEMPT",
                    &payload.idempotency_key,
                    trace_id,
                )
                .await;

            return Err(AppError::bad_request(
                "INSUFFICIENT_FUNDS",
                "source account has insufficient available balance",
            ));
        }

        let transfer = self
            .transfer_repository
            .create_transfer(CreateTransferInput {
                customer_id: payload.customer_id,
                from_account_id: payload.from_account_id,
                to_account_id: to_account_id.clone(),
                amount: payload.amount,
                currency: apply_result.currency,
                idempotency_key: payload.idempotency_key,
                status: "COMPLETED".to_string(),
            })
            .await;

        self.audit_repository
            .append(
                "CUSTOMER",
                &transfer.customer_id,
                "TRANSFER_CREATED",
                "TRANSFER",
                &transfer.transfer_id,
                trace_id,
            )
            .await;

        Ok(self.to_create_response(transfer))
    }

    pub async fn get_transfer_by_id(
        &self,
        transfer_id: &str,
        trace_id: Option<String>,
    ) -> Result<TransferDetailResponse, AppError> {
        let transfer = self
            .transfer_repository
            .find_by_id(transfer_id)
            .await
            .ok_or_else(|| {
                AppError::not_found("TRANSFER_NOT_FOUND", "transfer_id was not found")
            })?;

        self.audit_repository
            .append(
                "SYSTEM",
                "transfer-service",
                "TRANSFER_VIEWED",
                "TRANSFER",
                &transfer.transfer_id,
                trace_id,
            )
            .await;

        Ok(TransferDetailResponse {
            transfer_id: transfer.transfer_id,
            customer_id: transfer.customer_id,
            from_account_id: transfer.from_account_id,
            to_account_id: transfer.to_account_id,
            amount: transfer.amount,
            currency: transfer.currency,
            status: transfer.status,
            idempotency_key: transfer.idempotency_key,
            created_at: transfer.created_at,
        })
    }

    pub async fn list_transfers(
        &self,
        query: ListTransfersQuery,
        trace_id: Option<String>,
    ) -> Result<Vec<TransferSummaryResponse>, AppError> {
        let limit = validators::validate_list_transfers(&query)?;

        let transfers = self
            .transfer_repository
            .list(
                query.customer_id.as_deref(),
                query.account_id.as_deref(),
                limit,
            )
            .await;

        let entity_id = query
            .customer_id
            .as_deref()
            .or(query.account_id.as_deref())
            .unwrap_or("n/a");
        self.audit_repository
            .append(
                "SYSTEM",
                "transfer-service",
                "TRANSFER_HISTORY_VIEWED",
                "TRANSFER_QUERY",
                entity_id,
                trace_id,
            )
            .await;

        Ok(transfers
            .into_iter()
            .map(|transfer| TransferSummaryResponse {
                transfer_id: transfer.transfer_id,
                customer_id: transfer.customer_id,
                from_account_id: transfer.from_account_id,
                to_account_id: transfer.to_account_id,
                amount: transfer.amount,
                currency: transfer.currency,
                status: transfer.status,
                created_at: transfer.created_at,
            })
            .collect())
    }

    async fn resolve_beneficiary_account(
        &self,
        beneficiary_username: &str,
    ) -> Result<String, AppError> {
        let customer = self
            .fetch_customer_by_username(beneficiary_username)
            .await
            .map_err(|_| {
                AppError::not_found(
                    "BENEFICIARY_NOT_FOUND",
                    "beneficiary username does not exist",
                )
            })?;

        let default_account = self
            .fetch_default_account(&customer.id)
            .await
            .map_err(|_| {
                AppError::not_found(
                    "BENEFICIARY_NO_ACCOUNT",
                    "beneficiary has no default account",
                )
            })?;

        Ok(default_account.account_id)
    }

    async fn fetch_customer_by_username(
        &self,
        username: &str,
    ) -> Result<CustomerResponse, reqwest::Error> {
        let url = format!(
            "{}/api/v1/customers/by-username",
            self.user_service_base_url.trim_end_matches('/')
        );

        self.http_client
            .get(url)
            .query(&[("username", username)])
            .send()
            .await?
            .error_for_status()?
            .json::<CustomerResponse>()
            .await
    }

    async fn fetch_default_account(
        &self,
        customer_id: &str,
    ) -> Result<DefaultAccountResponse, reqwest::Error> {
        let url = format!(
            "{}/api/v1/accounts/default",
            self.account_service_base_url.trim_end_matches('/')
        );

        self.http_client
            .get(url)
            .query(&[("customer_id", customer_id)])
            .send()
            .await?
            .error_for_status()?
            .json::<DefaultAccountResponse>()
            .await
    }

    async fn fetch_account_balance(
        &self,
        account_id: &str,
    ) -> Result<AccountBalanceResponse, reqwest::Error> {
        let url = format!(
            "{}/api/v1/accounts/balance",
            self.account_service_base_url.trim_end_matches('/')
        );

        self.http_client
            .get(url)
            .query(&[("account_id", account_id)])
            .send()
            .await?
            .error_for_status()?
            .json::<AccountBalanceResponse>()
            .await
    }

    async fn apply_account_transfer(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: f64,
    ) -> Result<AccountApplyTransferResponse, reqwest::Error> {
        let url = format!(
            "{}/api/v1/accounts/apply-transfer",
            self.account_service_base_url.trim_end_matches('/')
        );

        self.http_client
            .post(url)
            .json(&AccountApplyTransferRequest {
                from_account_id: from_account_id.to_string(),
                to_account_id: to_account_id.to_string(),
                amount,
            })
            .send()
            .await?
            .error_for_status()?
            .json::<AccountApplyTransferResponse>()
            .await
    }

    fn to_create_response(&self, transfer: Transfer) -> CreateTransferResponse {
        CreateTransferResponse {
            transfer_id: transfer.transfer_id,
            status: transfer.status,
            customer_id: transfer.customer_id,
            from_account_id: transfer.from_account_id,
            to_account_id: transfer.to_account_id,
            amount: transfer.amount,
            currency: transfer.currency,
            created_at: transfer.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;

    use crate::{
        config::env::AppConfig,
        models::dto::{CreateTransferRequest, ListTransfersQuery},
        repositories::{
            audit_repository::AuditRepository, transfer_repository::TransferRepository,
        },
        services::transfer_service::TransferService,
    };

    #[tokio::test]
    async fn list_requires_one_filter() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost:5432/canbankx_transfer")
            .expect("failed to create lazy pool for tests");

        let service = TransferService::new(
            AppConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                database_url: "postgres://postgres:postgres@localhost:5432/canbankx_transfer".to_string(),
                account_service_base_url: "http://127.0.0.1:65535".to_string(),
                user_service_base_url: "http://127.0.0.1:65536".to_string(),
            },
            TransferRepository::new(pool.clone()),
            AuditRepository::new(pool),
        );

        let result = service
            .list_transfers(
                ListTransfersQuery {
                    customer_id: None,
                    account_id: None,
                    limit: Some(10),
                },
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn create_transfer_payload_validation() {
        let payload = CreateTransferRequest {
            customer_id: "".to_string(),
            from_account_id: "a1".to_string(),
            beneficiary_username: "testuser".to_string(),
            amount: 0.0,
            idempotency_key: "".to_string(),
        };

        assert!(crate::utils::validators::validate_create_transfer(&payload).is_err());
    }
}
