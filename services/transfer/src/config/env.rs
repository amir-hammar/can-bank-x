#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub account_service_base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let account_service_base_url = std::env::var("ACCOUNT_SERVICE_BASE_URL")
            .unwrap_or_else(|_| "http://account-service:8080".to_string());

        Self {
            host,
            port,
            account_service_base_url,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
