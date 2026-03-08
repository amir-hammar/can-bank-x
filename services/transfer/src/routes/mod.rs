use axum::Router;

use crate::app_state::AppState;

pub mod health_routes;
pub mod transfer_routes;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(health_routes::routes())
        .merge(transfer_routes::routes())
        .with_state(state)
}
