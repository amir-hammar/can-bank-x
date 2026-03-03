use std::env;

use sqlx::postgres::PgPoolOptions;
use user_service::{models::domain::AppState, router};

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required for user-service");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    let state = AppState { pool };
    let app = router::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind user-service");

    axum::serve(listener, app)
        .await
        .expect("user-service server error");
}
