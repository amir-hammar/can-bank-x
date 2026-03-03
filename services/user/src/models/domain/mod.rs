pub mod customer;
pub mod kyc_case;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}
