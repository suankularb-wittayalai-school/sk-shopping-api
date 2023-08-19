use std::fmt::Display;

use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FilterConfig, PaginationConfig, SortingConfig};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{FromRow, Type};

use super::request::{QueryableOrder, SortableOrder};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    NotShippedOut,
    Pending,
    Canceled,
    Delivered,
}

// impl OrderStatus {
//     pub fn to_string(&self) -> String {
//         match self {
//             Self::NotShippedOut => "not_shipped_out".to_string(),
//             Self::Pending => "pending".to_string(),
//             Self::Delivered => "delivered".to_string(),
//         }
//     }
// }

impl Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NotShippedOut => "not_shipped_out",
            Self::Pending => "pending",
            Self::Canceled => "canceled",
            Self::Delivered => "delivered",
        };
        write!(f, "{}", s)
    }
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::NotShippedOut
    }
}

impl Type<sqlx::Postgres> for OrderStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // <String as Type>::type_info()
        sqlx::postgres::PgTypeInfo::with_name("order_status")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for OrderStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let s: String = self.to_string();
        <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for OrderStatus {
    fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: String = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "not_shipped_out" => Ok(Self::NotShippedOut),
            "pending" => Ok(Self::Pending),
            "delivered" => Ok(Self::Delivered),
            "canceled" => Ok(Self::Canceled),
            _ => Err("invalid order status".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    SchoolPickup,
    Delivery,
}

impl Display for DeliveryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SchoolPickup => "pick_up",
            Self::Delivery => "delivery",
        };
        write!(f, "{}", s)
    }
}

impl Type<sqlx::Postgres> for DeliveryType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // <String as Type>::type_info()
        sqlx::postgres::PgTypeInfo::with_name("delivery_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DeliveryType {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let s: String = self.to_string();
        <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for DeliveryType {
    fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: String = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "pick_up" => Ok(Self::SchoolPickup),
            "delivery" => Ok(Self::Delivery),
            _ => Err("invalid delivery type".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cod,
    Promptpay,
    // Kplus,
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Cod => "cod",
            Self::Promptpay => "promptpay",
            // Self::Kplus => "mobile_banking_kbank",
        };
        write!(f, "{}", s)
    }
}

impl Type<sqlx::Postgres> for PaymentMethod {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // <String as Type>::type_info()
        sqlx::postgres::PgTypeInfo::with_name("payment_method")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for PaymentMethod {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let s: String = self.to_string();
        <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for PaymentMethod {
    fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let s: String = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "cod" => Ok(Self::Cod),
            "promptpay" => Ok(Self::Promptpay),
            // "kplus" => Ok(Self::Kplus),
            _ => Err("invalid payment method".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderTable {
    pub id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub buyer_id: Option<sqlx::types::Uuid>,
    pub is_paid: bool,
    pub is_verified: bool,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub zip_code: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub shipment_status: OrderStatus,
    pub delivery_type: DeliveryType,
    pub receiver_name: String,
    pub total_price: i64,
    pub payment_method: PaymentMethod,
    pub payment_slip_url: Option<String>,
    pub contact_email: String,
    pub contact_phone_number: Option<String>,
    pub ref_id: String,
    pub qr_code_file: Option<String>,
}

impl OrderTable {
    pub async fn get_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM orders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM orders
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;
        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT * FROM orders".to_string()
    }

    fn get_count_query() -> String {
        "SELECT COUNT(*) FROM orders".to_string()
    }

    fn append_where_clause<'a>(
        query: &mut String,
        filter: &'a FilterConfig<QueryableOrder>,
        params_count: i32,
    ) -> (
        i32,
        (
            Vec<String>,
            Vec<sqlx::types::Uuid>,
            Vec<&'a Vec<sqlx::types::Uuid>>,
            Vec<OrderStatus>,
            Vec<DeliveryType>,
            Vec<i64>,
            Vec<bool>,
        ),
    ) {
        let mut params_count = params_count;

        let mut string_params = Vec::new();
        let mut uuid_params = Vec::new();
        let mut uuid_array_params = Vec::new();
        let mut order_status_params = Vec::new();
        let mut delivery_type_params = Vec::new();
        let mut i64_params = Vec::new();
        let mut bool_params = Vec::new();

        if let Some(q) = &filter.q {
            if query.contains("WHERE") {
                query.push_str(&format!(
                    " AND (street_address_line_1 ILIKE ${} OR street_address_line_2 ILIKE ${} OR province ILIKE ${} OR district ILIKE ${} OR receiver_name ILIKE ${})",
                    params_count + 1,
                    params_count + 1,
                    params_count + 1,
                    params_count + 1,
                    params_count + 1
                ));
            } else {
                query.push_str(&format!(
                    " WHERE (street_address_line_1 ILIKE ${} OR street_address_line_2 ILIKE ${} OR province ILIKE ${} OR district ILIKE ${} OR receiver_name ILIKE ${})",
                    params_count + 1,
                    params_count + 1,
                    params_count + 1,
                    params_count + 1,
                    params_count + 1
                ));
            }

            string_params.push(format!("%{}%", q));
            params_count += 1;
        }

        if let Some(data) = &filter.data {
            if let Some(street_address_line_1) = &data.street_address_line_1 {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND street_address_line_1 ILIKE ${}",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE street_address_line_1 ILIKE ${}",
                        params_count + 1
                    ));
                }

                string_params.push(format!("%{}%", street_address_line_1));
                params_count += 1;
            }

            if let Some(street_address_line_2) = &data.street_address_line_2 {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND street_address_line_2 ILIKE ${}",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE street_address_line_2 ILIKE ${}",
                        params_count + 1
                    ));
                }

                string_params.push(format!("%{}%", street_address_line_2));
                params_count += 1;
            }

            if let Some(province) = &data.province {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND province ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE province ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", province));
                params_count += 1;
            }

            if let Some(district) = &data.district {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND district ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE district ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", district));
                params_count += 1;
            }

            if let Some(receiver_name) = &data.receiver_name {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND receiver_name ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE receiver_name ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", receiver_name));
                params_count += 1;
            }

            if let Some(id) = data.id {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id = ${}", params_count + 1));
                }

                uuid_params.push(id);
                params_count += 1;
            }

            if let Some(shop_ids) = &data.shop_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id IN (SELECT order_id FROM order_items INNER JOIN items ON order_items.item_id = items.id INNER JOIN listings ON items.listing_id = listings.id WHERE listings.shop_id = ANY(${}))", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id IN (SELECT order_id FROM order_items INNER JOIN items ON order_items.item_id = items.id INNER JOIN listings ON items.listing_id = listings.id WHERE listings.shop_id = ANY(${}))", params_count + 1));
                }

                uuid_array_params.push(shop_ids);
                params_count += 1;
            }

            if let Some(collection_ids) = &data.collection_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id IN (SELECT order_id FROM order_items INNER JOIN items ON order_items.item_id = items.id INNER JOIN listings ON items.listing_id = listings.id INNER JOIN collection_listings ON listings.id = collection_listings.listing_id WHERE collection_listings.collection_id = ANY(${}))", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id IN (SELECT order_id FROM order_items INNER JOIN items ON order_items.item_id = items.id INNER JOIN listings ON items.listing_id = listings.id INNER JOIN collection_listings ON listings.id = collection_listings.listing_id WHERE collection_listings.collection_id = ANY(${}))", params_count + 1));
                }

                uuid_array_params.push(collection_ids);
                params_count += 1;
            }

            if let Some(item_ids) = &data.item_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND id IN (SELECT order_id FROM order_items WHERE item_id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE id IN (SELECT order_id FROM order_items WHERE item_id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(item_ids);
                params_count += 1;
            }

            if let Some(buyer_ids) = &data.buyer_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND buyer_id = ANY(${})", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE buyer_id = ANY(${})", params_count + 1));
                }

                uuid_array_params.push(buyer_ids);
                params_count += 1;
            }

            if let Some(order_status) = data.shipping_status {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND shipping_status = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE shipping_status = ${}", params_count + 1));
                }

                order_status_params.push(order_status);
                params_count += 1;
            }

            if let Some(delivery_type) = data.delivery_type {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND delivery_type = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE delivery_type = ${}", params_count + 1));
                }

                delivery_type_params.push(delivery_type);
                params_count += 1;
            }

            if let Some(zip_code) = data.zip_code {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND zip_code = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE zip_code = ${}", params_count + 1));
                }

                i64_params.push(zip_code);
                params_count += 1;
            }

            if let Some(is_paid) = data.is_paid {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND is_paid = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE is_paid = ${}", params_count + 1));
                }

                bool_params.push(is_paid);
                params_count += 1;
            }

            if let Some(is_verified) = data.is_verified {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND is_verified = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE is_verified = ${}", params_count + 1));
                }

                bool_params.push(is_verified);
                params_count += 1;
            }
        }

        (
            params_count,
            (
                string_params,
                uuid_params,
                uuid_array_params,
                order_status_params,
                delivery_type_params,
                i64_params,
                bool_params,
            ),
        )
    }

    fn append_order_clause(query: &mut String, order: &Option<SortingConfig<SortableOrder>>) {
        let order = match order {
            Some(order) => order,
            None => return,
        };

        let sort_vec = match order.by.is_empty() {
            true => vec![SortableOrder::Id],
            false => order.by.clone(),
        };

        if !sort_vec.is_empty() {
            query.push_str(" ORDER BY ");

            let mut first = true;
            for s in sort_vec {
                if !first {
                    query.push_str(", ");
                }

                match s {
                    SortableOrder::Id => query.push_str("id"),
                    SortableOrder::CreatedAt => query.push_str("created_at"),
                    SortableOrder::BuyerId => query.push_str("buyer_id"),
                    SortableOrder::IsPaid => query.push_str("is_paid"),
                    SortableOrder::IsVerified => query.push_str("is_verified"),
                    SortableOrder::ShippingStatus => query.push_str("shipping_status"),
                }

                first = false;
            }

            match order.ascending {
                Some(true) => query.push_str(" ASC"),
                Some(false) => query.push_str(" DESC"),
                None => query.push_str(" ASC"),
            }
        }
    }

    fn append_limit_clause(
        query: &mut String,
        pagination: &Option<PaginationConfig>,
        params_count: i32,
    ) -> (i32, Vec<u32>) {
        let mut params_count = params_count;
        let mut params = Vec::new();

        let pagination = match pagination {
            Some(pagination) => pagination,
            None => &PaginationConfig {
                p: 0,
                size: Some(50),
            },
        };

        if let Some(size) = pagination.size {
            query.push_str(&format!(" LIMIT ${}", params_count + 1));
            params.push(size);
            params_count += 1;
        }

        query.push_str(&format!(" OFFSET ${}", params_count + 1));
        params.push(pagination.p);
        params_count += 1;

        (params_count, params)
    }

    pub async fn query(
        pool: &sqlx::PgPool,
        filter: &Option<FilterConfig<QueryableOrder>>,
        sorting: &Option<SortingConfig<SortableOrder>>,
        pagination: &Option<PaginationConfig>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = Self::get_default_query();

        let (
            params_count,
            (
                string_params,
                uuid_params,
                uuid_array_params,
                order_status_params,
                delivery_type_params,
                i64_params,
                bool_params,
            ),
        ) = if let Some(filter) = filter {
            Self::append_where_clause(&mut query, filter, 0)
        } else {
            (
                0,
                (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            )
        };

        // Sorting
        Self::append_order_clause(&mut query, sorting);

        // Pagination
        let (_params_count, pagination_params) =
            Self::append_limit_clause(&mut query, pagination, params_count);

        let mut query_builder = sqlx::query_as::<_, Self>(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        for param in uuid_params {
            query_builder = query_builder.bind(param);
        }

        for param in uuid_array_params {
            query_builder = query_builder.bind(param);
        }

        for param in order_status_params {
            query_builder = query_builder.bind(param);
        }

        for param in delivery_type_params {
            query_builder = query_builder.bind(param);
        }

        for param in i64_params {
            query_builder = query_builder.bind(param);
        }

        for param in bool_params {
            query_builder = query_builder.bind(param);
        }

        for param in pagination_params {
            query_builder = query_builder.bind(param as i64);
        }

        let result = query_builder.fetch_all(pool).await?;

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderItemTable {
    pub id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub order_id: sqlx::types::Uuid,
    pub item_id: sqlx::types::Uuid,
    pub amount: i64,
}

impl OrderItemTable {
    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM order_items
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;
        Ok(result)
    }
}
