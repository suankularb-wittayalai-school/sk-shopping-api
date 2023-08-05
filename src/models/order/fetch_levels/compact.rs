use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::order::db::{DeliveryType, OrderStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactOrder {
    pub id: uuid::Uuid,
    pub is_paid: bool,
    pub shipment_status: OrderStatus,
    pub total_price: i64,
    pub delivery_type: DeliveryType,
}

impl CompactOrder {
    pub async fn from_table(
        pool: &PgPool,
        order: super::super::db::OrderTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: order.id,
            is_paid: order.is_paid,
            shipment_status: order.shipment_status,
            // TODO: implement total_price
            total_price: 0,
            delivery_type: order.delivery_type,
        })
    }
}
