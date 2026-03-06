use crate::{
    models::dto::{AccountBalanceQuery, CreateAccountRequest, ListAccountsQuery},
    utils::errors::AppError,
};

pub fn is_valid_account_type(value: &str) -> bool {
    matches!(value, "CHEQUING" | "SAVINGS")
}

pub fn validate_create_account_payload(payload: &CreateAccountRequest) -> Result<(), AppError> {
    if payload.customer_id.trim().is_empty() || !is_valid_account_type(&payload.account_type) {
        return Err(AppError::bad_request(
            "INVALID_ACCOUNT_PAYLOAD",
            "customer_id and valid account_type are required",
        ));
    }

    if let Some(initial_balance) = payload.initial_balance {
        if !initial_balance.is_finite() || initial_balance < 0.0 {
            return Err(AppError::bad_request(
                "INVALID_INITIAL_BALANCE",
                "initial_balance must be a finite non-negative number",
            ));
        }
    }

    Ok(())
}

pub fn validate_list_accounts_query(query: &ListAccountsQuery) -> Result<(), AppError> {
    if query.customer_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_CUSTOMER_ID",
            "customer_id query parameter is required",
        ));
    }

    Ok(())
}

pub fn validate_balance_query(query: &AccountBalanceQuery) -> Result<(), AppError> {
    if query.account_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_ACCOUNT_ID",
            "account_id query parameter is required",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_type_validation() {
        assert!(is_valid_account_type("CHEQUING"));
        assert!(is_valid_account_type("SAVINGS"));
        assert!(!is_valid_account_type("INVALID"));
    }
}
