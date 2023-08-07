use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ListingTable {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub shop_id: sqlx::types::Uuid,
    pub is_hidden: bool,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub lifetime_stock: Option<i64>,
    pub amount_sold: Option<i64>,
}

impl ListingTable {
    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
    ) -> Result<Self, sqlx::Error> {
        let query = format!(
            "{} WHERE listings.id = $1 GROUP BY listings.id",
            Self::get_default_query()
        );

        let result = sqlx::query_as::<_, Self>(&query)
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(
            "{} WHERE id = ANY($1) GROUP BY listings.id",
            Self::get_default_query()
        );

        let result = sqlx::query_as::<_, Self>(&query)
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT listings.*, MIN(price) as price, MIN(discounted_price) as discounted_price, CAST(SUM(stock_added) as INT8) as lifetime_stock, CAST(SUM(amount) as INT8) as amount_sold, MIN(preorder_start) as preorder_start, MAX(preorder_end) as preorder_end
        FROM listings 
        INNER JOIN items ON listings.id = items.listing_id 
        LEFT JOIN item_stock_updates ON item_stock_updates.item_id = items.id
        LEFT JOIN order_items ON order_items.item_id = items.id
        ".to_string()
    }
}
