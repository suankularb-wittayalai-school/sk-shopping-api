use chrono::NaiveDateTime;
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};

use super::{item::Item, listing::Listing, shop::Shop};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyCollection {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
    pub shop: Shop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
    pub shop: Shop,
    pub items: Vec<Item>,
    pub listings: Vec<Listing>,
}

impl From<db::CollectionTable> for IdOnlyCollection {
    fn from(collection: db::CollectionTable) -> Self {
        Self { id: collection.id }
    }
}

impl From<db::CollectionTable> for CompactCollection {
    fn from(collection: db::CollectionTable) -> Self {
        Self {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            foreground_url: collection.foreground_url,
            background_url: collection.background_url,
        }
    }
}

impl DefaultCollection {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        collection: db::CollectionTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        // let shop = Shop::from_table(pool, shop, descendant_fetch_level)?;
        Ok(Self {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            foreground_url: collection.foreground_url,
            background_url: collection.background_url,
            // TODO: get shop from database
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

impl DetailedCollection {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        collection: db::CollectionTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        // let shop = Shop::from_table(pool, shop, descendant_fetch_level)?;
        Ok(Self {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            foreground_url: collection.foreground_url,
            background_url: collection.background_url,
            // TODO: get shop, items, listings from database
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable::default(),
                descendant_fetch_level,
                Some(&FetchLevel::IdOnly),
            )
            .await?,
            items: Vec::new(),
            listings: Vec::new(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum Collection {
    IdOnly(IdOnlyCollection),
    Compact(CompactCollection),
    Default(DefaultCollection),
    Detailed(DetailedCollection),
}

impl Serialize for Collection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Collection::IdOnly(collection) => collection.serialize(serializer),
            Collection::Compact(collection) => collection.serialize(serializer),
            Collection::Default(collection) => collection.serialize(serializer),
            Collection::Detailed(collection) => collection.serialize(serializer),
        }
    }
}

impl Collection {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        collection: db::CollectionTable,
        fetch_level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        match fetch_level {
            Some(FetchLevel::IdOnly) => Ok(Collection::IdOnly(collection.into())),
            Some(FetchLevel::Compact) => Ok(Collection::Compact(collection.into())),
            Some(FetchLevel::Default) => Ok(Collection::Default(
                DefaultCollection::from_table(pool, collection, descendant_fetch_level).await?,
            )),
            Some(FetchLevel::Detailed) => Ok(Collection::Detailed(
                DetailedCollection::from_table(pool, collection, descendant_fetch_level).await?,
            )),
            None => Ok(Collection::IdOnly(collection.into())),
        }
    }
}
