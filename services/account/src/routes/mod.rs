use axum::Router;

use crate::app_state::AppState;

pub mod account_routes;
pub mod health_routes;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(health_routes::routes())
        .merge(account_routes::routes())
        .with_state(state)
}
