use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FetchLevel, FilterConfig, PaginationConfig};
use parallel_stream::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use self::request::QueryableItem;

use super::{collection::Collection, listing::Listing, shop::Shop};

pub(crate) mod db;
pub(crate) mod request;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyItem {
    pub id: sqlx::types::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactItem {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub colors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultItem {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub colors: Vec<String>,
    pub images_url: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedItem {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub colors: Vec<String>,
    pub images_url: Vec<String>,
    pub shop: Shop,
    pub listing: Listing,
    pub collections: Vec<Collection>,
}

impl From<db::ItemTable> for IdOnlyItem {
    fn from(item: db::ItemTable) -> Self {
        Self { id: item.id }
    }
}

impl CompactItem {
    pub async fn from_table(pool: &sqlx::PgPool, item: db::ItemTable) -> Result<Self, sqlx::Error> {
        // get lifetime_stock from item_stock_updates
        let lifetime_stock = sqlx::query(
            r#"
            SELECT CAST(SUM(stock_added) as INT8) as lifetime_stock FROM item_stock_updates WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let lifetime_stock = lifetime_stock
            .get::<Option<i64>, _>("lifetime_stock")
            .unwrap_or(0);

        // get amount_sold from order_items
        let amount_sold = sqlx::query(
            r#"
            SELECT CAST(SUM(amount) as INT8) as amount_sold FROM order_items WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let amount_sold = amount_sold
            .get::<Option<i64>, _>("amount_sold")
            .unwrap_or(0);

        // get colors from item_colors
        let colors = sqlx::query(
            r#"
            SELECT color FROM item_colors WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;

        let colors: Vec<String> = colors
            .into_iter()
            .map(|row| row.get::<String, _>("color"))
            .collect();

        Ok(CompactItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            lifetime_stock,
            amount_sold,
            colors,
        })
    }
}

impl DefaultItem {
    pub async fn from_table(pool: &sqlx::PgPool, item: db::ItemTable) -> Result<Self, sqlx::Error> {
        // get lifetime_stock from item_stock_updates
        let lifetime_stock = sqlx::query(
            r#"
            SELECT CAST(SUM(stock_added) as INT8) as lifetime_stock FROM item_stock_updates WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let lifetime_stock = lifetime_stock
            .get::<Option<i64>, _>("lifetime_stock")
            .unwrap_or(0);

        // get amount_sold from order_items
        let amount_sold = sqlx::query(
            r#"
            SELECT CAST(SUM(amount) as INT8) as amount_sold FROM order_items WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let amount_sold = amount_sold
            .get::<Option<i64>, _>("amount_sold")
            .unwrap_or(0);

        // get colors from item_colors
        let colors = sqlx::query(
            r#"
            SELECT color FROM item_colors WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;

        let colors: Vec<String> = colors
            .into_iter()
            .map(|row| row.get::<String, _>("color"))
            .collect();

        // get images_url from item_images
        let images_url = sqlx::query(
            r#"
            SELECT image_url FROM item_images WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;

        let images_url: Vec<String> = images_url
            .into_iter()
            .map(|row| row.get::<String, _>("image_url"))
            .collect();

        Ok(DefaultItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            preorder_start: item.preorder_start,
            preorder_end: item.preorder_end,
            lifetime_stock,
            amount_sold,
            colors,
            images_url,
        })
    }
}

impl DetailedItem {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        item: db::ItemTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        // get lifetime_stock from item_stock_updates
        let lifetime_stock = sqlx::query(
            r#"
            SELECT CAST(SUM(stock_added) as INT8) as lifetime_stock FROM item_stock_updates WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let lifetime_stock = lifetime_stock
            .get::<Option<i64>, _>("lifetime_stock")
            .unwrap_or(0);

        // get amount_sold from order_items
        let amount_sold = sqlx::query(
            r#"
            SELECT CAST(SUM(amount) as INT8) as amount_sold FROM order_items WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_one(pool)
        .await?;

        let amount_sold = amount_sold
            .get::<Option<i64>, _>("amount_sold")
            .unwrap_or(0);

        // get colors from item_colors
        let colors = sqlx::query(
            r#"
            SELECT color FROM item_colors WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;

        let colors: Vec<String> = colors
            .into_iter()
            .map(|row| row.get::<String, _>("color"))
            .collect();

        // get images_url from item_images
        let images_url = sqlx::query(
            r#"
            SELECT image_url FROM item_images WHERE item_id = $1
            "#,
        )
        .bind(item.id)
        .fetch_all(pool)
        .await?;

        let images_url: Vec<String> = images_url
            .into_iter()
            .map(|row| row.get::<String, _>("image_url"))
            .collect();

        let collections = sqlx::query(
            r#"
            SELECT collection_id FROM collection_listings WHERE listing_id = $1
            "#,
        )
        .bind(item.listing_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<Uuid, _>("collection_id"))
        .collect();

        let collections = Collection::get_by_ids(
            pool,
            collections,
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        let shop_id = sqlx::query(
            r#"
            SELECT shop_id FROM listings WHERE id = $1
            "#,
        )
        .bind(item.listing_id)
        .fetch_one(pool)
        .await?
        .get::<Uuid, _>("shop_id");

        Ok(DetailedItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            lifetime_stock,
            amount_sold,
            preorder_start: item.preorder_start,
            preorder_end: item.preorder_end,
            colors,
            images_url,
            listing: Listing::get_by_id(
                pool,
                item.listing_id,
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            collections,
            shop: Shop::get_by_id(
                pool,
                shop_id,
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum Item {
    IdOnly(IdOnlyItem),
    Compact(CompactItem),
    Default(DefaultItem),
    Detailed(DetailedItem),
}

impl Serialize for Item {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Item::IdOnly(item) => item.serialize(serializer),
            Item::Compact(item) => item.serialize(serializer),
            Item::Default(item) => item.serialize(serializer),
            Item::Detailed(item) => item.serialize(serializer),
        }
    }
}

impl Item {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        item: db::ItemTable,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        match level {
            Some(FetchLevel::IdOnly) => Ok(Item::IdOnly(IdOnlyItem::from(item))),
            Some(FetchLevel::Compact) => {
                Ok(Item::Compact(CompactItem::from_table(pool, item).await?))
            }
            Some(FetchLevel::Default) => {
                Ok(Item::Default(DefaultItem::from_table(pool, item).await?))
            }
            Some(FetchLevel::Detailed) => Ok(Item::Detailed(
                DetailedItem::from_table(pool, item, descendant_fetch_level).await?,
            )),
            _ => Ok(Item::IdOnly(IdOnlyItem::from(item))),
        }
    }

    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let item = db::ItemTable::get_by_id(pool, id).await?;
        Self::from_table(pool, item, level, descendant_fetch_level).await
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let items = db::ItemTable::get_by_ids(pool, ids).await?;
        // let result = items
        //     .into_par_stream()
        //     .map(|item| async move {
        //         let data = Self::from_table(pool, item, level, descendant_fetch_level).await;
        //         match data {
        //             Ok(data) => Some(data),
        //             Err(_) => None,
        //         }
        //     })
        //     .collect::<Vec<_>>()
        //     .await;

        // parallel stream is not working due to lifetime issue
        let mut result = vec![];
        for item in items {
            let data = Self::from_table(pool, item, level, descendant_fetch_level).await?;
            result.push(data);
        }
        Ok(result)
    }

    pub async fn query(
        pool: &sqlx::PgPool,
        filter: &Option<FilterConfig<QueryableItem>>,
        pagination: &Option<PaginationConfig>,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let items = db::ItemTable::query(pool, filter, pagination).await?;
        // let result = items
        //     .into_par_stream()
        //     .map(|item| async move {
        //         let data = Self::from_table(pool, item, level, descendant_fetch_level).await;
        //         match data {
        //             Ok(data) => Some(data),
        //             Err(_) => None,
        //         }
        //     })
        //     .collect::<Vec<_>>()
        //     .await;

        // parallel stream is not working due to lifetime issue
        let mut result = vec![];
        for item in items {
            let data = Self::from_table(pool, item, level, descendant_fetch_level).await?;
            result.push(data);
        }
        Ok(result)
    }
}
