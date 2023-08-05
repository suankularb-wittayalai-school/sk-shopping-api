use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::Row;

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
            variants_id,
            Some(&FetchLevel::Compact),
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        // get price being the lowest price of all variants
        let price = variants
            .iter()
            .map(|item| match item {
                Item::Compact(item) => item.price,
                Item::Default(item) => item.price,
                Item::Detailed(item) => item.price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        let discounted_price = variants
            .iter()
            .map(|item| match item {
                Item::Compact(item) => item.discounted_price,
                Item::Default(item) => item.discounted_price,
                Item::Detailed(item) => item.discounted_price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        // get is_sold_out being the sum of all variants lifetime_stock - amount_sold
        let is_sold_out = variants
            .iter()
            .map(|item| match item {
                Item::Compact(item) => item.lifetime_stock - item.amount_sold,
                Item::Default(item) => item.lifetime_stock - item.amount_sold,
                Item::Detailed(item) => item.lifetime_stock - item.amount_sold,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .sum::<i64>()
            <= 0;

        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            thumbnail_url: listing.thumbnail_url,
            price,
            discounted_price,
            is_sold_out,
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

        // get default variants for other fields
        let default_variants =
            Item::get_by_ids(pool, variants_id, None, Some(&FetchLevel::IdOnly)).await?;

        // get price being the lowest price of all variants
        let price = default_variants
            .iter()
            .map(|item| match item {
                Item::Compact(item) => item.price,
                Item::Default(item) => item.price,
                Item::Detailed(item) => item.price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        let discounted_price = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.discounted_price,
                Item::Detailed(item) => item.discounted_price,
                Item::Compact(item) => item.discounted_price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        // get preorder_start being the earliest preorder_start of all variants
        let preorder_start = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.preorder_start,
                Item::Detailed(item) => item.preorder_start,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        // get preorder_end being the latest preorder_end of all variants
        let preorder_end = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.preorder_end,
                Item::Detailed(item) => item.preorder_end,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .max()
            .unwrap_or_default();

        // get lifetime_stock being the sum of all variants lifetime_stock
        let lifetime_stock = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.lifetime_stock,
                Item::Detailed(item) => item.lifetime_stock,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .sum::<i64>();

        // get amount_sold being the sum of all variants amount_sold
        let amount_sold = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.amount_sold,
                Item::Detailed(item) => item.amount_sold,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .sum::<i64>();

        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            variants,
            preorder_start,
            preorder_end,
            price,
            discounted_price,
            lifetime_stock,
            amount_sold,
            // TODO: get shop stock values from db
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
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

        // get default variants for other fields
        let default_variants =
            Item::get_by_ids(pool, variants_id, None, Some(&FetchLevel::IdOnly)).await?;

        // get price being the lowest price of all variants
        let price = default_variants
            .iter()
            .map(|item| match item {
                Item::Compact(item) => item.price,
                Item::Default(item) => item.price,
                Item::Detailed(item) => item.price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        let discounted_price = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.discounted_price,
                Item::Detailed(item) => item.discounted_price,
                Item::Compact(item) => item.discounted_price,
                Item::IdOnly(_item) => unreachable!("Item::IdOnly should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        // get preorder_start being the earliest preorder_start of all variants
        let preorder_start = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.preorder_start,
                Item::Detailed(item) => item.preorder_start,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .min()
            .unwrap_or_default();

        // get preorder_end being the latest preorder_end of all variants
        let preorder_end = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.preorder_end,
                Item::Detailed(item) => item.preorder_end,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .max()
            .unwrap_or_default();

        // get lifetime_stock being the sum of all variants lifetime_stock
        let lifetime_stock = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.lifetime_stock,
                Item::Detailed(item) => item.lifetime_stock,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .sum::<i64>();

        // get amount_sold being the sum of all variants amount_sold
        let amount_sold = default_variants
            .iter()
            .map(|item| match item {
                Item::Default(item) => item.amount_sold,
                Item::Detailed(item) => item.amount_sold,
                _ => unreachable!("Item::IdOnly and Item::Compact should not be in variants"),
            })
            .sum::<i64>();

        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            variants,
            preorder_start,
            preorder_end,
            price,
            discounted_price,
            lifetime_stock,
            amount_sold,
            // TODO: get shop collections values from db
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            collections: vec![],
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
}
