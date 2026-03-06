use account_service::{app_state::AppState, config::env::AppConfig, create_app};

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    let state = AppState::new();
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(config.address())
        .await
        .expect("failed to bind account-service");

    axum::serve(listener, app)
        .await
        .expect("account-service server error");
}
