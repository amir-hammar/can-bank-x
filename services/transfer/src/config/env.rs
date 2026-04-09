#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub account_service_base_url: String,
    pub user_service_base_url: String,
    pub central_bank_enabled: bool,
    pub central_bank_participant_id: String,
    pub central_bank_participant_name: String,
    pub central_bank_bootstrap_servers: String,
    pub central_bank_topic: String,
    pub central_bank_payment_service_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is required for transfer-service");

        let account_service_base_url = std::env::var("ACCOUNT_SERVICE_BASE_URL")
            .unwrap_or_else(|_| "http://account-service:8080".to_string());

        let user_service_base_url = std::env::var("USER_SERVICE_BASE_URL")
            .unwrap_or_else(|_| "http://user-service:8080".to_string());

        let central_bank_enabled = std::env::var("CENTRAL_BANK_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let central_bank_participant_id = std::env::var("CENTRAL_BANK_PARTICIPANT_ID")
            .unwrap_or_else(|_| "AmirBank".to_string());

        let central_bank_participant_name = std::env::var("CENTRAL_BANK_PARTICIPANT_NAME")
            .unwrap_or_else(|_| "AmirBank".to_string());

        let central_bank_bootstrap_servers = std::env::var("CENTRAL_BANK_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "10.194.32.170:9094".to_string());

        let central_bank_topic = std::env::var("CENTRAL_BANK_TOPIC")
            .unwrap_or_else(|_| "canbankx-events".to_string());

        let central_bank_payment_service_url = std::env::var("CENTRAL_BANK_PAYMENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://payment-service:8080".to_string());

        Self {
            host,
            port,
            database_url,
            account_service_base_url,
            user_service_base_url,
            central_bank_enabled,
            central_bank_participant_id,
            central_bank_participant_name,
            central_bank_bootstrap_servers,
            central_bank_topic,
            central_bank_payment_service_url,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
