use unzip_n::unzip_n;

use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::models::{
    auth::user::User,
    order::{
        db::{DeliveryType, OrderStatus},
        OrderItem,
    },
};

unzip_n!(pub 3);

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultOrder {
    pub id: sqlx::types::Uuid,
    pub is_paid: bool,
    pub shipment_status: OrderStatus,
    pub total_price: i64,
    pub delivery_type: DeliveryType,
    pub items: Vec<OrderItem>,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub zip_code: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub pickup_location: Option<Vec<String>>,
    pub buyer: Option<User>,
    pub receiver_name: String,
}

impl DefaultOrder {
    pub async fn from_table(
        pool: &PgPool,
        order: super::super::db::OrderTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let items_db = sqlx::query(
            r#"
            SELECT id, item_id
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
        let (order_items_id, items_id): (Vec<sqlx::types::Uuid>, Vec<sqlx::types::Uuid>) = items_db
            .into_iter()
            .map(|row| {
                (
                    row.get::<sqlx::types::Uuid, _>("id"),
                    row.get::<sqlx::types::Uuid, _>("item_id"),
                )
            })
            .unzip();

        let items =
            OrderItem::get_by_ids(pool, order_items_id.clone(), descendant_fetch_level).await?;

        let pickup_location = sqlx::query(
            r#"
            SELECT pickup_location
            FROM shops INNER JOIN listings ON shops.id = listings.shop_id
            WHERE listings.id = ANY($1)
            "#,
        )
        .bind(&items_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<Option<String>, _>("pickup_location"))
        .collect::<Vec<Option<String>>>();

        let pickup_location = pickup_location
            .iter()
            .filter_map(|pickup_location| pickup_location.clone())
            .collect::<Vec<String>>();

        let pickup_location = if pickup_location.is_empty() {
            None
        } else {
            Some(pickup_location)
        };

        let user = if order.buyer_id.is_some() {
            Some(User::from_id(order.buyer_id.unwrap(), pool, descendant_fetch_level).await?)
        } else {
            None
        };

        Ok(Self {
            id: order.id,
            is_paid: order.is_paid,
            shipment_status: order.shipment_status,
            delivery_type: order.delivery_type,
            items,
            street_address_line_1: order.street_address_line_1,
            street_address_line_2: order.street_address_line_2,
            zip_code: order.zip_code,
            province: order.province,
            district: order.district,
            pickup_location,
            buyer: user,
            receiver_name: order.receiver_name,
            total_price: order.total_price,
        })
    }
}
