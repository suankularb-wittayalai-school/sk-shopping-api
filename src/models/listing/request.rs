use serde::{Deserialize, Serialize};

use crate::models::common::RangeQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableListing {
    pub name: Option<String>,
    pub description: Option<String>,
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub is_hidden: Option<bool>,
    pub price_range: Option<RangeQuery>,
    pub stock_range: Option<RangeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableListing {
    Id,
    Name,
    CreatedAt,
    Stock,
    Price,
}
