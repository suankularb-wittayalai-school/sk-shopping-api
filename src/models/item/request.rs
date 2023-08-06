use serde::{Deserialize, Serialize};

use crate::models::common::RangeQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableItem {
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub name: Option<String>,
    pub stock_range: Option<RangeQuery>,
    pub price_range: Option<RangeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortableItem {
    Id,
    Name,
    Stock,
    Price,
}
