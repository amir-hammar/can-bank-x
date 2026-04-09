use axum::Router;

use crate::app_state::AppState;

pub mod central_bank_routes;
pub mod health_routes;
pub mod transfer_routes;

pub fn create_router(state: AppState) -> Router {
    let router = Router::new()
        .merge(health_routes::routes())
        .merge(transfer_routes::routes());

    let router = if state.central_bank_service.is_some() {
        router.merge(central_bank_routes::central_bank_routes())
    } else {
        router
    };

    router.with_state(state)
}
