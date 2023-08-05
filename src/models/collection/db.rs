use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct CollectionTable {
    pub id: uuid::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
    pub shop_id: sqlx::types::Uuid,
}
