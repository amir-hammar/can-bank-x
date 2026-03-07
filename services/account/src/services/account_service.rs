use crate::{
    models::dto::{
        AccountBalanceQuery, AccountBalanceResponse, AccountSummaryResponse,
        ApplyTransferRequest, ApplyTransferResponse, CreateAccountRequest, CreateAccountResponse,
        ListAccountsQuery,
    },
    repositories::{
        account_repository::{AccountRepository, ApplyTransferRepoError},
        audit_repository::AuditRepository,
    },
    utils::{errors::AppError, validators},
};

#[derive(Clone)]
pub struct AccountService {
    repository: AccountRepository,
    audit_repository: AuditRepository,
}

impl AccountService {
    pub fn new(repository: AccountRepository, audit_repository: AuditRepository) -> Self {
        Self {
            repository,
            audit_repository,
        }
    }

    pub async fn create_account(
        &self,
        payload: CreateAccountRequest,
        trace_id: Option<String>,
    ) -> Result<CreateAccountResponse, AppError> {
        validators::validate_create_account_payload(&payload)?;

        let account = self.repository.create_account(&payload).await;

        self.audit_repository
            .append(
                "CUSTOMER",
                &payload.customer_id,
                "ACCOUNT_CREATED",
                "ACCOUNT",
                &account.account_id,
                trace_id,
            )
            .await;

        Ok(CreateAccountResponse {
            account_id: account.account_id,
            status: "created",
            account_type: account.account_type,
            currency: account.currency,
            available_balance: account.available_balance,
        })
    }

    pub async fn list_accounts(
        &self,
        query: ListAccountsQuery,
        trace_id: Option<String>,
    ) -> Result<Vec<AccountSummaryResponse>, AppError> {
        validators::validate_list_accounts_query(&query)?;

        let accounts = self
            .repository
            .list_accounts_by_customer(&query.customer_id)
            .await;

        self.audit_repository
            .append(
                "CUSTOMER",
                &query.customer_id,
                "ACCOUNTS_LIST_VIEWED",
                "CUSTOMER",
                &query.customer_id,
                trace_id,
            )
            .await;

        Ok(accounts
            .into_iter()
            .map(|account| AccountSummaryResponse {
                account_id: account.account_id,
                customer_id: account.customer_id,
                account_type: account.account_type,
                status: account.status,
                currency: account.currency,
                available_balance: account.available_balance,
            })
            .collect())
    }

    pub async fn get_balance(
        &self,
        query: AccountBalanceQuery,
        trace_id: Option<String>,
    ) -> Result<AccountBalanceResponse, AppError> {
        validators::validate_balance_query(&query)?;

        let account = self
            .repository
            .get_account_by_id(&query.account_id)
            .await
            .ok_or_else(|| AppError::not_found("ACCOUNT_NOT_FOUND", "account_id was not found"))?;

        self.audit_repository
            .append(
                "SYSTEM",
                "account-service",
                "ACCOUNT_BALANCE_VIEWED",
                "ACCOUNT",
                &account.account_id,
                trace_id,
            )
            .await;

        Ok(AccountBalanceResponse {
            account_id: account.account_id,
            available_balance: account.available_balance,
            ledger_balance: account.ledger_balance,
            currency: account.currency,
        })
    }

    pub async fn apply_transfer(
        &self,
        payload: ApplyTransferRequest,
        trace_id: Option<String>,
    ) -> Result<ApplyTransferResponse, AppError> {
        validators::validate_apply_transfer_payload(&payload)?;

        let (from_account, to_account) = self
            .repository
            .apply_transfer(
                &payload.from_account_id,
                &payload.to_account_id,
                payload.amount,
            )
            .await
            .map_err(|error| match error {
                ApplyTransferRepoError::SourceAccountNotFound => {
                    AppError::not_found("ACCOUNT_NOT_FOUND", "from_account_id was not found")
                }
                ApplyTransferRepoError::DestinationAccountNotFound => {
                    AppError::not_found("ACCOUNT_NOT_FOUND", "to_account_id was not found")
                }
                ApplyTransferRepoError::CurrencyMismatch => AppError::bad_request(
                    "CURRENCY_MISMATCH",
                    "source and destination accounts must have same currency",
                ),
                ApplyTransferRepoError::InsufficientFunds => AppError::bad_request(
                    "INSUFFICIENT_FUNDS",
                    "source account has insufficient available balance",
                ),
            })?;

        self.audit_repository
            .append(
                "SYSTEM",
                "account-service",
                "ACCOUNT_TRANSFER_APPLIED",
                "ACCOUNT_TRANSFER",
                &format!(
                    "{}->{}",
                    payload.from_account_id, payload.to_account_id
                ),
                trace_id,
            )
            .await;

        Ok(ApplyTransferResponse {
            from_account_id: payload.from_account_id,
            to_account_id: payload.to_account_id,
            amount: payload.amount,
            currency: from_account.currency,
            from_available_balance: from_account.available_balance,
            to_available_balance: to_account.available_balance,
        })
    }
}
