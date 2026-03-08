use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::{
    auth_controller::me as auth_me,
    customer_controller::{me as customer_me, register as customer_register},
    health_controller::{health, metrics},
    kyc_controller::{confirm_kyc, kyc_status, submit_kyc},
};
use crate::models::domain::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/api/v1/customers/register", post(customer_register))
        .route("/api/v1/customers/me", get(customer_me))
        .route("/api/v1/auth/me", get(auth_me))
        .route("/api/v1/kyc/submit", post(submit_kyc))
        .route("/api/v1/kyc/confirm", post(confirm_kyc))
        .route("/api/v1/kyc/status", get(kyc_status))
        .with_state(state)
}
