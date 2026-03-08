use axum::{routing::get, routing::post, Router};

use crate::controllers::transfer_controller;

pub fn routes() -> Router<crate::app_state::AppState> {
    Router::new()
        .route(
            "/api/v1/transfers",
            post(transfer_controller::create_transfer),
        )
        .route(
            "/api/v1/transfers",
            get(transfer_controller::list_transfers),
        )
        .route(
            "/api/v1/transfers/:id",
            get(transfer_controller::get_transfer_by_id),
        )
}
