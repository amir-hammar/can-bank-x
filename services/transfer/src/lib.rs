pub mod app_state;
pub mod cache;
pub mod config;
pub mod controllers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use axum::Router;

pub fn create_app(state: AppState) -> Router {
    routes::create_router(state)
}
