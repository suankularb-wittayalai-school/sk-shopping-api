use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ListingTable {
    pub id: String,
    pub created_at: Option<NaiveDateTime>,
    pub name: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
}
