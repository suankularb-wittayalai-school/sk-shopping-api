use serde::{Deserialize, Serialize};

use crate::models::order::db::{DeliveryType, OrderStatus, OrderTable};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactOrder {
    pub id: sqlx::types::Uuid,
    pub receiver_name: String,
    pub is_paid: bool,
    pub shipment_status: OrderStatus,
    pub total_price: i64,
    pub delivery_type: DeliveryType,
}

impl From<OrderTable> for CompactOrder {
    fn from(order: OrderTable) -> Self {
        Self {
            id: order.id,
            receiver_name: order.receiver_name,
            is_paid: order.is_paid,
            shipment_status: order.shipment_status,
            total_price: order.total_price,
            delivery_type: order.delivery_type,
        }
    }
}
