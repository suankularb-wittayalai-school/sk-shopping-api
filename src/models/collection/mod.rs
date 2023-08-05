use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::{item::Item, listing::Listing, shop::Shop};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyCollection {
    pub id: sqlx::types::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactCollection {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultCollection {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
    pub shop: Shop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedCollection {
    pub id: sqlx::types::Uuid,
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
    #[async_recursion]
    pub async fn from_table<'a: 'async_recursion>(
        pool: &sqlx::PgPool,
        collection: db::CollectionTable,
        descendant_fetch_level: Option<&'a FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        // let shop = Shop::from_table(pool, shop, descendant_fetch_level)?;

        let listing_ids = sqlx::query(
            r#"
            SELECT listing_id FROM collection_listings
            WHERE collection_id = $1
            "#,
        )
        .bind(collection.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<sqlx::types::Uuid, _>("listing_id"))
        .collect::<Vec<_>>();

        let listings = Listing::get_by_ids(
            pool,
            listing_ids.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        let items_ids = sqlx::query(
            r#"
            SELECT id FROM items WHERE listing_id = ANY($1)
            "#,
        )
        .bind(&listing_ids)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<sqlx::types::Uuid, _>("id"))
        .collect::<Vec<_>>();

        let items = Item::get_by_ids(
            pool,
            items_ids.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        // let items = Item::get_by_ids(pool, ids, level, descendant_fetch_level)

        Ok(Self {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            foreground_url: collection.foreground_url,
            background_url: collection.background_url,
            items,
            listings,
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
