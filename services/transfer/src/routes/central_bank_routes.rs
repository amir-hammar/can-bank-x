use axum::{routing::{delete, get, post}, Router};

use crate::controllers::central_bank_controller;

pub fn central_bank_routes() -> Router<crate::app_state::AppState> {
    Router::new()
        // Alias endpoints
        .route(
            "/api/v1/central-bank/aliases",
            get(central_bank_controller::list_aliases)
                .post(central_bank_controller::register_alias),
        )
        .route(
            "/api/v1/central-bank/aliases/lookup",
            get(central_bank_controller::lookup_alias),
        )
        .route(
            "/api/v1/central-bank/aliases/:alias",
            delete(central_bank_controller::delete_alias),
        )
        // Payment endpoints
        .route(
            "/api/v1/central-bank/payments",
            get(central_bank_controller::list_payments)
                .post(central_bank_controller::initiate_payment),
        )
        // Alias transfer endpoints
        .route(
            "/api/v1/central-bank/alias-transfers",
            post(central_bank_controller::request_alias_transfer),
        )
        .route(
            "/api/v1/central-bank/alias-transfers/pending",
            get(central_bank_controller::list_pending_transfers),
        )
        .route(
            "/api/v1/central-bank/alias-transfers/:transfer_id/approve",
            post(central_bank_controller::approve_transfer),
        )
        .route(
            "/api/v1/central-bank/alias-transfers/:transfer_id/deny",
            post(central_bank_controller::deny_transfer),
        )
        // Settlement endpoint
        .route(
            "/api/v1/central-bank/settlement",
            get(central_bank_controller::get_settlement),
        )
        // Status endpoint
        .route(
            "/api/v1/central-bank/status",
            get(central_bank_controller::status),
        )
}
