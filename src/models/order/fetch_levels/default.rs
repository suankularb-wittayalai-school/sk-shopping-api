use std::vec;

use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::{
    auth::user::{User, UserTable},
    item::Item,
    order::db::{DeliveryType, OrderStatus},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultOrder {
    pub id: sqlx::types::Uuid,
    pub is_paid: bool,
    pub shipment_status: OrderStatus,
    pub total_price: i64,
    pub delivery_type: DeliveryType,
    pub items: Vec<Item>,
    pub shipping_address_line_1: Option<String>,
    pub shipping_address_line_2: Option<String>,
    pub zip_code: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub pickup_location: Option<String>,
    pub buyer: User,
}

impl DefaultOrder {
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
            // TODO: implement items
            items: vec![],
            shipping_address_line_1: order.shipping_address_line_1,
            shipping_address_line_2: order.shipping_address_line_2,
            zip_code: order.zip_code,
            province: order.province,
            district: order.district,
            pickup_location: order.pickup_location,
            // TODO: implement buyer
            buyer: User::from_table(
                UserTable {
                    id: order.buyer_id,
                    username: "".to_string(),
                    email: "".to_string(),
                    profile: Some("".to_string()),
                    first_name: Some("".to_string()),
                    last_name: Some("".to_string()),
                    created_at: None,
                },
                descendant_fetch_level,
            )
            .await,
        })
    }
}
