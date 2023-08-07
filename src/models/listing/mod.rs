use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use mysk_lib::models::common::{requests::FetchLevel, string::MultiLangString};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use super::{collection::Collection, item::Item, shop::Shop};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyListing {
    pub id: sqlx::types::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactListing {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: String,
    pub shop: Shop,
    pub is_sold_out: bool,
    pub thumbnail_url: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultListing {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: String,
    pub shop: Shop,
    pub thumbnail_url: Option<String>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub variants: Vec<Item>,
    pub categories: Vec<MultiLangString>,
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedListing {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: String,
    pub shop: Shop,
    pub thumbnail_url: Option<String>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub variants: Vec<Item>,
    pub collections: Vec<Collection>,
    pub categories: Vec<MultiLangString>,
    pub is_hidden: bool,
}

impl From<db::ListingTable> for IdOnlyListing {
    fn from(listing: db::ListingTable) -> Self {
        Self { id: listing.id }
    }
}

impl CompactListing {
    #[async_recursion]
    pub async fn from_table<'a: 'async_recursion>(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&'a FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            is_hidden: listing.is_hidden,
            shop: Shop::get_by_id(
                pool,
                listing.shop_id,
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            thumbnail_url: listing.thumbnail_url,
            price: listing.price,
            discounted_price: listing.discounted_price,
            is_sold_out: listing.lifetime_stock.unwrap_or(0) - listing.amount_sold.unwrap_or(0)
                == 0,
        })
    }
}

impl DefaultListing {
    #[async_recursion]
    pub async fn from_table<'a: 'async_recursion>(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&'a FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let variants_id = sqlx::query(
            r#"
            SELECT * FROM items
            WHERE listing_id = $1
            "#,
        )
        .bind(listing.id)
        .fetch_all(pool)
        .await?;

        let variants_id = variants_id
            .into_iter()
            .map(|item| item.get("id"))
            .collect::<Vec<sqlx::types::Uuid>>();

        let variants = Item::get_by_ids(
            pool,
            variants_id.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        let categories = sqlx::query(
            r#"
            SELECT * FROM listing_categories INNER JOIN categories ON listing_categories.category_id = categories.id
            WHERE listing_id = $1 
            "#,
        )
        .bind(listing.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|item| MultiLangString::new(item.get("name_th"), item.get("name_en")))
        .collect::<Vec<MultiLangString>>();

        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            is_hidden: listing.is_hidden,
            variants,
            preorder_start: listing.preorder_start,
            preorder_end: listing.preorder_end,
            price: listing.price,
            discounted_price: listing.discounted_price,
            lifetime_stock: listing.lifetime_stock.unwrap_or(0),
            amount_sold: listing.amount_sold.unwrap_or(0),
            categories,
            shop: Shop::get_by_id(
                pool,
                listing.shop_id,
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
        })
    }
}

impl DetailedListing {
    #[async_recursion]
    pub async fn from_table<'a: 'async_recursion>(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&'a FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let variants_id = sqlx::query(
            r#"
            SELECT * FROM items 
            WHERE listing_id = $1 
            "#,
        )
        .bind(listing.id)
        .fetch_all(pool)
        .await?;

        let variants_id = variants_id
            .into_iter()
            .map(|item| item.get("id"))
            .collect::<Vec<sqlx::types::Uuid>>();

        let variants = Item::get_by_ids(
            pool,
            variants_id.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        let categories = sqlx::query(
            r#"
            SELECT * FROM listing_categories INNER JOIN categories ON listing_categories.category_id = categories.id
            WHERE listing_id = $1
            "#,
        )
        .bind(listing.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|item| MultiLangString::new(item.get("name_th"), item.get("name_en")))
        .collect::<Vec<MultiLangString>>();

        let collections = sqlx::query(
            r#"
            SELECT collection_id FROM collection_listings WHERE listing_id = $1
            "#,
        )
        .bind(listing.id)
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

        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            is_hidden: listing.is_hidden,
            variants,
            preorder_start: listing.preorder_start,
            preorder_end: listing.preorder_end,
            price: listing.price,
            discounted_price: listing.discounted_price,
            lifetime_stock: listing.lifetime_stock.unwrap_or(0),
            amount_sold: listing.amount_sold.unwrap_or(0),
            categories,
            shop: Shop::get_by_id(
                pool,
                listing.shop_id,
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            collections,
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum Listing {
    IdOnly(IdOnlyListing),
    Compact(CompactListing),
    Default(DefaultListing),
    Detailed(DetailedListing),
}

impl Serialize for Listing {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Listing::IdOnly(listing) => listing.serialize(serializer),
            Listing::Compact(listing) => listing.serialize(serializer),
            Listing::Default(listing) => listing.serialize(serializer),
            Listing::Detailed(listing) => listing.serialize(serializer),
        }
    }
}

impl Listing {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        fetch_level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        match fetch_level {
            Some(FetchLevel::IdOnly) => Ok(Listing::IdOnly(IdOnlyListing::from(listing))),
            Some(FetchLevel::Compact) => Ok(Listing::Compact(
                CompactListing::from_table(pool, listing, descendant_fetch_level).await?,
            )),
            Some(FetchLevel::Default) => Ok(Listing::Default(
                DefaultListing::from_table(pool, listing, descendant_fetch_level).await?,
            )),
            Some(FetchLevel::Detailed) => Ok(Listing::Detailed(
                DetailedListing::from_table(pool, listing, descendant_fetch_level).await?,
            )),
            None => Ok(Listing::IdOnly(IdOnlyListing::from(listing))),
        }
    }

    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let listing = db::ListingTable::get_by_id(pool, id).await?;

        Self::from_table(pool, listing, level, descendant_fetch_level).await
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let listings = db::ListingTable::get_by_ids(pool, ids).await?;

        let mut result = Vec::with_capacity(listings.len());

        for listing in listings {
            result.push(Self::from_table(pool, listing, level, descendant_fetch_level).await?);
        }

        Ok(result)
    }
}
