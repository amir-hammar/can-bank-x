use account_service::{app_state::AppState, config::env::AppConfig, create_app};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState::new(pool).await;
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(config.address())
        .await
        .expect("failed to bind account-service");

    axum::serve(listener, app)
        .await
        .expect("account-service server error");
}
