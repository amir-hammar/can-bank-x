use crate::{
    models::dto::{
        AccountBalanceQuery, AccountBalanceResponse, AccountSummaryResponse, CreateAccountRequest,
        CreateAccountResponse, ListAccountsQuery,
    },
    repositories::account_repository::AccountRepository,
    utils::{errors::AppError, validators},
};

#[derive(Clone)]
pub struct AccountService {
    repository: AccountRepository,
}

impl AccountService {
    pub fn new(repository: AccountRepository) -> Self {
        Self { repository }
    }

    pub async fn create_account(
        &self,
        payload: CreateAccountRequest,
    ) -> Result<CreateAccountResponse, AppError> {
        validators::validate_create_account_payload(&payload)?;

        let account = self.repository.create_account(&payload).await;

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
    ) -> Result<Vec<AccountSummaryResponse>, AppError> {
        validators::validate_list_accounts_query(&query)?;

        let accounts = self
            .repository
            .list_accounts_by_customer(&query.customer_id)
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
    ) -> Result<AccountBalanceResponse, AppError> {
        validators::validate_balance_query(&query)?;

        let account = self
            .repository
            .get_account_by_id(&query.account_id)
            .await
            .ok_or_else(|| {
                AppError::not_found("ACCOUNT_NOT_FOUND", "account_id was not found")
            })?;

        Ok(AccountBalanceResponse {
            account_id: account.account_id,
            available_balance: account.available_balance,
            ledger_balance: account.ledger_balance,
            currency: account.currency,
        })
    }
}
