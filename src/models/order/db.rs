use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{FromRow, Type};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    NotShippedOut,
    Pending,
    Delivered,
}

impl OrderStatus {
    pub fn to_string(&self) -> String {
        match self {
            Self::NotShippedOut => "not_shipped_out".to_string(),
            Self::Pending => "pending".to_string(),
            Self::Delivered => "delivered".to_string(),
        }
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
            _ => Err("invalid order status".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    SchoolPickup,
    Delivery,
}

impl DeliveryType {
    pub fn to_string(&self) -> String {
        match self {
            Self::SchoolPickup => "school_pickup".to_string(),
            Self::Delivery => "delivery".to_string(),
        }
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderTable {
    pub id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub buyer_id: sqlx::types::Uuid,
    pub is_paid: bool,
    pub street_address_line_1: Option<String>,
    pub street_address_line_2: Option<String>,
    pub zip_code: Option<String>,
    pub province: Option<String>,
    pub district: Option<String>,
    pub shipment_status: OrderStatus,
    pub delivery_type: DeliveryType,
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
