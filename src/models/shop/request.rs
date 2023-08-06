use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableShop {
    pub id: Option<sqlx::types::Uuid>,
    pub name: Option<String>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub manager_ids: Option<Vec<sqlx::types::Uuid>>,
    pub accept_promptpay: Option<bool>,
    pub accept_cod: Option<bool>,
    pub is_school_pickup_allowed: Option<bool>,
    pub is_delivery_allowed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableShop {
    Id,
    NameTh,
    NameEn,
    CreatedAt,
}
