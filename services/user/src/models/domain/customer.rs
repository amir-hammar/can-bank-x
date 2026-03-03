use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Customer {
    pub id: String,
    pub keycloak_sub: String,
    pub email: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CustomerProfile {
    pub customer_id: String,
    pub full_name: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub nas: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CustomerWithProfile {
    pub customer: Customer,
    pub profile: CustomerProfile,
}
