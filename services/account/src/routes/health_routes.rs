use axum::{routing::get, Router};

use crate::controllers::health_controller;

pub fn routes() -> Router<crate::app_state::AppState> {
    Router::new()
        .route("/health", get(health_controller::health))
        .route("/metrics", get(health_controller::metrics))
}
