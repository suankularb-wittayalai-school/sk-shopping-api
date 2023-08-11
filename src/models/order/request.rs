use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::models::address::Address;

use super::{
    db::{DeliveryType, OrderStatus, PaymentMethod},
    omise,
};

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
    payment_method: PaymentMethod,
}

impl CreatableOrder {
    pub async fn insert(
        &self,
        pool: &sqlx::PgPool,
        omise_secret_key: &str,
        user_id: Option<Uuid>,
    ) -> Result<Uuid, sqlx::Error> {
        let mut total_price = 0;

        let mut transaction = pool.begin().await?;

        for item in &self.items {
            let item_db = sqlx::query(
                r#"
                SELECT LEAST(price, discounted_price) AS price
                FROM items
                WHERE id = $1
                "#,
            )
            .bind(item.item_id)
            .fetch_one(transaction.as_mut())
            .await?;

            total_price += item_db.get::<i64, _>("price") * item.amount;
        }

        // create omise charge
        let omise_charge = match self.payment_method {
            PaymentMethod::Promptpay => {
                let res = omise::OmiseCharge::new(
                    total_price,
                    PaymentMethod::Promptpay,
                    omise_secret_key,
                )
                .await;

                match res {
                    Ok(omise_charge) => Some(omise_charge),
                    Err(_) => None,
                }
            }
            // PaymentMethod::Kplus => {
            //     let res =
            //         omise::OmiseCharge::new(total_price, PaymentMethod::Kplus, omise_secret_key)
            //             .await;

            //     match res {
            //         Ok(omise_charge) => Some(omise_charge),
            //         Err(_) => None,
            //     }
            // }
            _ => None,
        };

        let (street_address_line_1, street_address_line_2, province, district, zip_code) =
            match &self.address {
                Some(address) => (
                    Some(address.street_address_line_1.clone()),
                    address.street_address_line_2.clone(),
                    Some(address.province.clone()),
                    Some(address.district.clone()),
                    Some(address.zip_code),
                ),
                None => (None, None, None, None, None),
            };

        // create order
        let order_id = sqlx::query(
            r#"
            INSERT INTO orders (buyer_id, street_address_line_1, street_address_line_2, province, district, zip_code, delivery_type, receiver_name, payment_method, total_price, omise_charge_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(street_address_line_1)
        .bind(street_address_line_2)
        .bind(province)
        .bind(district)
        .bind(zip_code)
        .bind(self.delivery_type)
        .bind(self.receiver_name.clone())
        .bind(self.payment_method)
        .bind(total_price)
        .bind(omise_charge.map(|omise_charge| omise_charge.id))
        .fetch_one(transaction.as_mut())
        .await?
        .get::<sqlx::types::Uuid, _>("id");

        // create order items
        for item in &self.items {
            sqlx::query(
                r#"
                INSERT INTO order_items (order_id, item_id, amount)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(order_id)
            .bind(item.item_id)
            .bind(item.amount)
            .execute(transaction.as_mut())
            .await?;
        }

        transaction.commit().await?;

        Ok(order_id)
    }
}
