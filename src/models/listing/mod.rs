use chrono::NaiveDateTime;
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};

use super::{collection::Collection, item::Item, shop::Shop};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyListing {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactListing {
    pub id: String,
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
    pub id: String,
    pub name: String,
    pub description: String,
    pub shop: Shop,
    pub thumbnail_url: Option<String>,
    pub preorder_start: Option<NaiveDateTime>,
    pub preorder_end: Option<NaiveDateTime>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub variants: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedListing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub shop: Shop,
    pub thumbnail_url: Option<String>,
    pub preorder_start: Option<NaiveDateTime>,
    pub preorder_end: Option<NaiveDateTime>,
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
    pub async fn from_table(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
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
            // TODO: get price and stock values from db
            is_sold_out: false,
            price: 0,
            discounted_price: None,
        })
    }
}

impl DefaultListing {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            // TODO: get shop price, stock, preorder, variants values from db
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            preorder_start: None,
            preorder_end: None,
            price: 0,
            discounted_price: None,
            lifetime_stock: 0,
            amount_sold: 0,
            variants: vec![],
        })
    }
}

impl DetailedListing {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        listing: db::ListingTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: listing.id,
            name: listing.name,
            description: listing.description,
            thumbnail_url: listing.thumbnail_url,
            // TODO: get shop price, stock, preorder, variants values from db
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            preorder_start: None,
            preorder_end: None,
            price: 0,
            discounted_price: None,
            lifetime_stock: 0,
            amount_sold: 0,
            variants: vec![],
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
