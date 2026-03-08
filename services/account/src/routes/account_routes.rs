use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::account_controller;

pub fn routes() -> Router<crate::app_state::AppState> {
    Router::new()
        .route(
            "/api/v1/accounts/create",
            post(account_controller::create_account),
        )
        .route("/api/v1/accounts", get(account_controller::list_accounts))
        .route(
            "/api/v1/accounts/balance",
            get(account_controller::get_balance),
        )
        .route(
            "/api/v1/accounts/default",
            get(account_controller::get_default_account),
        )
        .route(
            "/api/v1/accounts/apply-transfer",
            post(account_controller::apply_transfer),
        )
}
