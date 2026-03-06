pub mod customer;
pub mod kyc_case;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub kyc_mock_data_path: String,
}
