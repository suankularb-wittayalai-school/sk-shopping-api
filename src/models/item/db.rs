use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ItemTable {
    pub id: uuid::Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<NaiveDateTime>,
    pub preorder_end: Option<NaiveDateTime>,
    pub listing_id: Option<String>,
}
