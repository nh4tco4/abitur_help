use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Serialize, FromRow)]
pub struct FaqEntry {
    pub id: i32,
    pub question: String,
    pub answer: String,
    pub keywords: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct NewFaqEntry {
    pub question: String,
    pub keywords: String,
    pub answer: String,
}
