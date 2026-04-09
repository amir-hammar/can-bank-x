use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use transfer_service::{
    app_state::AppState,
    config::env::AppConfig,
    create_app,
    repositories::central_bank_repository::CentralBankRepository,
    services::{
        central_bank_service::CentralBankService,
        kafka_consumer::run_central_bank_consumer,
        kafka_producer::CentralBankKafkaProducer,
    },
};

#[tokio::main]
async fn main() {
    env_logger::init();

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

    // ── Central Bank integration (optional) ────────────────────────────────
    let central_bank_service = if config.central_bank_enabled {
        let producer = CentralBankKafkaProducer::new(
            &config.central_bank_bootstrap_servers,
            &config.central_bank_topic,
            &config.central_bank_participant_id,
        );

        let repo = CentralBankRepository::new(pool.clone());

        let svc = Arc::new(CentralBankService::new(
            producer.clone(),
            repo,
            config.central_bank_participant_id.clone(),
            config.central_bank_payment_service_url.clone(),
            config.account_service_base_url.clone(),
        ));

        // Spawn the Kafka consumer as a background task
        let consumer_config = config.clone();
        let consumer_pool = pool.clone();
        let consumer_producer = producer;
        tokio::spawn(async move {
            run_central_bank_consumer(consumer_config, consumer_pool, consumer_producer).await;
        });

        eprintln!(
            "Central bank integration ENABLED — participant={}",
            config.central_bank_participant_id
        );

        Some(svc)
    } else {
        eprintln!("Central bank integration DISABLED");
        None
    };

    let state = AppState::new(config.clone(), pool, central_bank_service).await;
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(config.address())
        .await
        .expect("failed to bind transfer-service");

    axum::serve(listener, app)
        .await
        .expect("transfer-service server error");
}
