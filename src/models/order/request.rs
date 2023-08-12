use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::models::address::Address;

use super::db::{DeliveryType, OrderStatus, PaymentMethod};

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
    payment_slip_url: Option<String>,
    contact_email: String,
    contact_phone_number: Option<String>,
}

impl CreatableOrder {
    pub async fn insert(
        &self,
        pool: &sqlx::PgPool,
        user_id: Option<Uuid>,
    ) -> Result<Uuid, sqlx::Error> {
        let mut total_price = 0;

        let mut transaction = pool.begin().await?;

        // make sure all the items are not out of stock and they are from the same shop
        let shop_ids = sqlx::query(
            r#"
            SELECT DISTINCT shop_id
            FROM items
            WHERE id = ANY($1)
            "#,
        )
        .bind(
            &self
                .items
                .iter()
                .map(|item| item.item_id)
                .collect::<Vec<sqlx::types::Uuid>>(),
        )
        .fetch_all(transaction.as_mut())
        .await?
        .into_iter()
        .map(|row| row.get::<sqlx::types::Uuid, _>("shop_id"))
        .collect::<Vec<sqlx::types::Uuid>>();

        if shop_ids.len() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }

        let curr_stock = sqlx::query(
            "SELECT
        CAST(SUM(stock_added) AS INT8) - CAST(SUM(amount) AS INT8) AS stock_left
      FROM
        items
        LEFT JOIN (
          SELECT
            item_id,
            SUM(stock_added) AS stock_added
          FROM item_stock_updates
          GROUP BY item_id
        ) AS stock_agg ON items.id = stock_agg.item_id
        LEFT JOIN (
          SELECT
            item_id,
            SUM(amount) AS amount
          FROM order_items WHERE order_id IN (
            SELECT id FROM orders WHERE NOT (shipment_status = 'canceled' OR (created_at > NOW() - INTERVAL '1 day' AND is_paid = FALSE))
          )
          GROUP BY item_id
        ) AS amount_agg ON items.id = amount_agg.item_id
      WHERE
        items.id = ANY($1)
      GROUP BY
        items.id",
        )
        .bind(
            &self
                .items
                .iter()
                .map(|item| item.item_id)
                .collect::<Vec<sqlx::types::Uuid>>(),
        )
        .fetch_all(transaction.as_mut())
        .await?
        .into_iter()
        .map(|row| row.get::<i64, _>("stock_left"))
        .collect::<Vec<i64>>();

        for (i, item) in self.items.iter().enumerate() {
            if curr_stock[i] < item.amount {
                return Err(sqlx::Error::RowNotFound);
            }
        }

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
            INSERT INTO orders (buyer_id, street_address_line_1, street_address_line_2, province, district, zip_code, delivery_type, receiver_name, payment_method, total_price, payment_slip_url, contact_email, contact_phone_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        .bind(self.payment_slip_url.clone())
        .bind(self.contact_email.clone())
        .bind(self.contact_phone_number.clone())
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
