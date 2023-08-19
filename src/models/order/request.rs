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
    pub buyer_ids: Option<Vec<sqlx::types::Uuid>>,
    pub shipping_status: Option<OrderStatus>,
    pub delivery_type: Option<DeliveryType>,
    pub receiver_name: Option<String>,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub zip_code: Option<i64>,
    pub is_paid: Option<bool>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableOrder {
    Id,
    CreatedAt,
    BuyerId,
    IsPaid,
    IsVerified,
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
            INNER JOIN listings ON items.listing_id = listings.id
            WHERE items.id = ANY($1)
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

        let shop_id = shop_ids[0];

        let curr_stock = sqlx::query(
            "SELECT
            CAST(SUM(stock_added) AS INT8) as life_time_stock_added,
            CAST(SUM(amount) AS INT8) as life_time_amount
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
            SELECT id FROM orders WHERE NOT (shipment_status = 'canceled' OR (created_at > NOW() - INTERVAL '3 minute' AND is_paid = FALSE))
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
        .map(|row| row.get::<Option<i64>, _>("life_time_stock_added").unwrap_or(0) - row.get::<Option<i64>, _>("life_time_amount").unwrap_or(0))
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

        let shipping_fee = match &self.delivery_type {
            DeliveryType::Delivery => 70,
            DeliveryType::SchoolPickup => 0,
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
            INSERT INTO orders (buyer_id, street_address_line_1, street_address_line_2, province, district, zip_code, delivery_type, receiver_name, payment_method, total_price, payment_slip_url, contact_email, contact_phone_number, shop_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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
        .bind(total_price + shipping_fee)
        .bind(self.payment_slip_url.clone())
        .bind(self.contact_email.clone())
        .bind(self.contact_phone_number.clone())
        .bind(shop_id)
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

    pub fn validate(&self) -> Result<&Self, String> {
        if self.items.is_empty() {
            return Err("items must not be empty".to_string());
        }

        if self.receiver_name.is_empty() {
            return Err("receiver_name must not be empty".to_string());
        }

        if self.contact_email.is_empty() {
            return Err("contact_email must not be empty".to_string());
        }

        // make sure email is valid
        let email_regex =
            regex::Regex::new(r"^\w+([\.-]?\w+)*@\w+([\.-]?\w+)*(\.\w{2,3})+$").unwrap();

        if !email_regex.is_match(&self.contact_email) {
            return Err("contact_email must be a valid email".to_string());
        }

        if self.delivery_type == DeliveryType::Delivery && self.address.is_none() {
            return Err("address must not be empty".to_string());
        }

        if self.delivery_type == DeliveryType::Delivery && self.address.is_some() {
            let address = self.address.as_ref().unwrap();

            if address.street_address_line_1.is_empty() {
                return Err("street_address_line_1 must not be empty".to_string());
            }

            if address.province.is_empty() {
                return Err("province must not be empty".to_string());
            }

            if address.district.is_empty() {
                return Err("district must not be empty".to_string());
            }

            if address.zip_code == 0 {
                return Err("zip_code must not be empty".to_string());
            }
        }

        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatableOrder {
    pub receiver_name: Option<String>,
    pub payment_slip_url: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone_number: Option<String>,
    pub is_paid: Option<bool>,
    pub is_verified: Option<bool>,
    pub shipment_status: Option<OrderStatus>,
}

impl UpdatableOrder {
    pub async fn commit_changes(
        &self,
        pool: &sqlx::PgPool,
        order_id: sqlx::types::Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut query = String::from("UPDATE orders SET ");
        let mut param_count = 1;

        let mut param_segments = Vec::new();
        let mut string_params = Vec::new();
        let mut bool_params = Vec::new();
        let mut order_status_params = Vec::new();

        if let Some(receiver_name) = &self.receiver_name {
            param_segments.push(format!("receiver_name = ${}", param_count));
            string_params.push(receiver_name);
            param_count += 1;
        }

        if let Some(payment_slip_url) = &self.payment_slip_url {
            param_segments.push(format!("payment_slip_url = ${}", param_count));
            string_params.push(payment_slip_url);
            param_count += 1;
        }

        if let Some(contact_email) = &self.contact_email {
            param_segments.push(format!("contact_email = ${}", param_count));
            string_params.push(contact_email);
            param_count += 1;
        }

        if let Some(contact_phone_number) = &self.contact_phone_number {
            param_segments.push(format!("contact_phone_number = ${}", param_count));
            string_params.push(contact_phone_number);
            param_count += 1;
        }

        if let Some(is_paid) = &self.is_paid {
            param_segments.push(format!("is_paid = ${}", param_count));
            bool_params.push(is_paid);
            param_count += 1;
        }

        if let Some(is_verified) = &self.is_verified {
            param_segments.push(format!("is_verified = ${}", param_count));
            bool_params.push(is_verified);
            param_count += 1;
        }

        if let Some(shipment_status) = &self.shipment_status {
            param_segments.push(format!("shipment_status = ${}", param_count));
            order_status_params.push(shipment_status);
            param_count += 1;
        }

        query.push_str(&param_segments.join(", "));

        query.push_str(" WHERE id = $");
        query.push_str(&param_count.to_string());

        let mut query_builder = sqlx::query(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        for param in bool_params {
            query_builder = query_builder.bind(param);
        }

        for param in order_status_params {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(order_id);

        query_builder.execute(pool).await?;

        Ok(())
    }
}
