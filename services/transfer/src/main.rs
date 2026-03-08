use sqlx::postgres::PgPoolOptions;
use transfer_service::{app_state::AppState, config::env::AppConfig, create_app};

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to transfer-service postgres database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run transfer-service migrations");

    let state = AppState::new(config.clone(), pool).await;
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(config.address())
        .await
        .expect("failed to bind transfer-service");

    axum::serve(listener, app)
        .await
        .expect("transfer-service server error");
}
