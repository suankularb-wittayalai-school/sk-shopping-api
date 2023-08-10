use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::address::Address;

use super::db::{DeliveryType, OrderStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableOrder {
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub buyer_id: Option<Vec<sqlx::types::Uuid>>,
    pub shipping_status: Option<OrderStatus>,
    pub delivery_type: Option<DeliveryType>,
    pub receiver_name: Option<String>,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub zip_code: Option<i64>,
    pub is_paid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableOrder {
    Id,
    CreatedAt,
    BuyerId,
    IsPaid,
    ShippingStatus, // TODO sort by price once the price is added to the order query
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAmount {
    pub item_id: sqlx::types::Uuid,
    pub amount: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatableOrder {
    items: Vec<ItemAmount>,
    delivery_type: DeliveryType,
    address: Option<Address>,
    receiver_name: String,
}

impl CreatableOrder {
    pub async fn insert(
        &self,
        pool: &sqlx::PgPool,
        omise_secret_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<Uuid, sqlx::Error> {
        todo!()
    }
}
