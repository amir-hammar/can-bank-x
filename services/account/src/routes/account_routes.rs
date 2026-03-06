use axum::{routing::{get, post}, Router};

use crate::controllers::account_controller;

pub fn routes() -> Router<crate::app_state::AppState> {
    Router::new()
        .route("/api/v1/accounts/create", post(account_controller::create_account))
        .route("/api/v1/accounts", get(account_controller::list_accounts))
        .route("/api/v1/accounts/balance", get(account_controller::get_balance))
}
