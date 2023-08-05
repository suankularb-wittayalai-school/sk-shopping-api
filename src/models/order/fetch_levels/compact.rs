use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::models::order::db::{DeliveryType, OrderStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactOrder {
    pub id: sqlx::types::Uuid,
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
        })
    }
}
