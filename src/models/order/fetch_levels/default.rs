use std::vec;

use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

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
        let items_db = sqlx::query(
            r#"
            SELECT item_id, amount
            FROM order_items
            WHERE order_id = $1
            "#,
        )
        .bind(order.id)
        .fetch_all(pool)
        .await?;

        // let items_id = items_db
        //     .into_iter()
        //     .map(|row| row.get::<sqlx::types::Uuid, _>("item_id"))
        // .collect::<Vec<sqlx::types::Uuid>>();
        let (items_id, item_amount): (Vec<sqlx::types::Uuid>, Vec<i32>) = items_db
            .into_iter()
            .map(|row| {
                (
                    row.get::<sqlx::types::Uuid, _>("item_id"),
                    row.get::<i32, _>("amount"),
                )
            })
            .unzip();

        let items = Item::get_by_ids(
            pool,
            items_id.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        let price_per_item = sqlx::query(
            r#"
            SELECT item_id, price, discounted_price
            FROM items
            WHERE id = ANY($1)
            "#,
        )
        .bind(&items_id)
        .fetch_all(pool)
        .await?;

        let total_price = price_per_item
            .into_iter()
            .map(|row| {
                let item_id = row.get::<sqlx::types::Uuid, _>("item_id");
                let price = row.get::<i64, _>("price");
                let discounted_price = row.get::<i64, _>("discounted_price");
                let amount = item_amount
                    .iter()
                    .find(|&&amount| items_id[amount as usize] == item_id)
                    .unwrap();
                if discounted_price == 0 {
                    price * *amount as i64
                } else {
                    discounted_price * *amount as i64
                }
            })
            .sum::<i64>();

        Ok(Self {
            id: order.id,
            is_paid: order.is_paid,
            shipment_status: order.shipment_status,
            total_price,
            delivery_type: order.delivery_type,
            items,
            shipping_address_line_1: order.shipping_address_line_1,
            shipping_address_line_2: order.shipping_address_line_2,
            zip_code: order.zip_code,
            province: order.province,
            district: order.district,
            // TODO: implement pickup_location based on delivery_type and shop
            pickup_location: Some("".to_string()),
            buyer: User::from_id(order.buyer_id, pool, descendant_fetch_level).await?,
        })
    }
}
