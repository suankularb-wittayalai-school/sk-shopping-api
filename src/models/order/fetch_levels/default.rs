use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::models::{
    auth::user::User,
    order::{
        db::{DeliveryType, OrderStatus, PaymentMethod},
        OrderItem,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultOrder {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub ref_id: String,
    pub is_paid: bool,
    pub is_verified: bool,
    pub shipment_status: OrderStatus,
    pub total_price: i64,
    pub delivery_type: DeliveryType,
    pub items: Vec<OrderItem>,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub zip_code: Option<i64>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub pickup_location: Option<Vec<String>>,
    pub buyer: Option<User>,
    pub receiver_name: String,
    pub payment_method: PaymentMethod,
    pub payment_slip_url: Option<String>,
    pub promptpay_qr_code_url: Option<String>,
    // pub qr_code_file: Option<String>,
    pub contact_email: String,
    pub contact_phone_number: Option<String>,
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

        // let promptpay_qr_code_url = match order.payment_method {
        //     PaymentMethod::Promptpay => {
        //         // get shop promptpay number and make sure it is not null
        //         let promptpay_number = sqlx::query(
        //             r#"
        //             SELECT promptpay_number
        //             FROM shops
        //             INNER JOIN listings ON shops.id = listings.shop_id
        //             INNER JOIN items ON listings.id = items.listing_id
        //             INNER JOIN order_items ON items.id = order_items.item_id
        //             WHERE order_items.order_id = $1
        //             "#,
        //         )
        //         .bind(order.id)
        //         .fetch_one(pool)
        //         .await?
        //         .get::<Option<String>, _>("promptpay_number");

        //         match promptpay_number {
        //             Some(promptpay_number) => {
        //                 let promptpay_qr_code_url = format!(
        //                     "https://promptpay.io/{}/{}.png",
        //                     promptpay_number, order.total_price
        //                 );

        //                 Some(promptpay_qr_code_url)
        //             }
        //             None => None,
        //         }
        //     }
        //     _ => None,
        // };

        Ok(Self {
            id: order.id,
            created_at: order.created_at,
            ref_id: order.ref_id,
            is_paid: order.is_paid,
            is_verified: order.is_verified,
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
            payment_method: order.payment_method,
            payment_slip_url: order.payment_slip_url,
            promptpay_qr_code_url: order.qr_code_file,
            contact_email: order.contact_email,
            contact_phone_number: order.contact_phone_number,
        })
    }
}
