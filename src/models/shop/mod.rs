use std::vec;

use async_recursion::async_recursion;
use mysk_lib::models::common::{
    requests::{FetchLevel, FilterConfig, PaginationConfig, SortingConfig},
    string::MultiLangString,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use self::{
    db::ShopTable,
    request::{QueryableShop, SortableShop},
};

use super::{collection::Collection, item::Item, listing::Listing};

pub(crate) mod db;
pub(crate) mod request;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyShop {
    pub id: sqlx::types::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactShop {
    pub id: sqlx::types::Uuid,
    pub name: MultiLangString,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultShop {
    pub id: sqlx::types::Uuid,
    pub name: MultiLangString,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
    pub logo_url: Option<String>,
    pub is_school_pickup_allowed: bool,
    pub pickup_location: Option<String>,
    pub is_delivery_allowed: bool,
    pub accept_promptpay: bool,
    pub promptpay_number: Option<String>,
    pub accept_cod: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedShop {
    pub id: sqlx::types::Uuid,
    pub name: MultiLangString,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
    pub logo_url: Option<String>,
    pub is_school_pickup_allowed: bool,
    pub pickup_location: Option<String>,
    pub is_delivery_allowed: bool,
    pub accept_promptpay: bool,
    pub promptpay_number: Option<String>,
    pub accept_cod: bool,
    pub listings: Vec<Listing>,
    pub collections: Vec<Collection>,
    pub items: Vec<Item>,
}

impl From<db::ShopTable> for IdOnlyShop {
    fn from(shop: db::ShopTable) -> Self {
        Self { id: shop.id }
    }
}

impl From<db::ShopTable> for CompactShop {
    fn from(shop: db::ShopTable) -> Self {
        Self {
            id: shop.id,
            name: MultiLangString::new(shop.name_en, shop.name_th),
            accent_color: shop.accent_color,
            background_color: shop.background_color,
            logo_url: shop.logo_url,
        }
    }
}

impl From<db::ShopTable> for DefaultShop {
    fn from(shop: db::ShopTable) -> Self {
        Self {
            id: shop.id,
            name: MultiLangString::new(shop.name_en, shop.name_th),
            accent_color: shop.accent_color,
            background_color: shop.background_color,
            logo_url: shop.logo_url,
            is_school_pickup_allowed: shop.is_school_pickup_allowed,
            pickup_location: shop.pickup_location,
            is_delivery_allowed: shop.is_delivery_allowed,
            accept_promptpay: shop.accept_promptpay,
            promptpay_number: shop.promptpay_number,
            accept_cod: shop.accept_cod,
        }
    }
}

impl DetailedShop {
    #[async_recursion]
    pub async fn from_table<'a: 'async_recursion>(
        pool: &sqlx::PgPool,
        shop: db::ShopTable,
        descendant_fetch_level: Option<&'a FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let listing_ids = sqlx::query(
            r#"
            SELECT id FROM listings WHERE shop_id = $1 ORDER BY priority DESC
            "#,
        )
        .bind(shop.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<sqlx::types::Uuid, _>("id"))
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

        let collections_id = sqlx::query(
            r#"
            SELECT id FROM collections WHERE shop_id = $1
            "#,
        )
        .bind(shop.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<sqlx::types::Uuid, _>("id"))
        .collect::<Vec<_>>();

        let collections = Collection::get_by_ids(
            pool,
            collections_id.clone(),
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        Ok(Self {
            id: shop.id,
            name: MultiLangString::new(shop.name_en, shop.name_th),
            accent_color: shop.accent_color,
            background_color: shop.background_color,
            logo_url: shop.logo_url,
            is_school_pickup_allowed: shop.is_school_pickup_allowed,
            pickup_location: shop.pickup_location,
            is_delivery_allowed: shop.is_delivery_allowed,
            accept_promptpay: shop.accept_promptpay,
            promptpay_number: shop.promptpay_number,
            accept_cod: shop.accept_cod,
            listings,
            items,
            collections,
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum Shop {
    IdOnly(IdOnlyShop),
    Compact(CompactShop),
    Default(DefaultShop),
    Detailed(DetailedShop),
}

impl Shop {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        shop: db::ShopTable,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        match level {
            Some(FetchLevel::IdOnly) => Ok(Self::IdOnly(shop.into())),
            Some(FetchLevel::Compact) => Ok(Self::Compact(shop.into())),
            Some(FetchLevel::Default) => Ok(Self::Default(shop.into())),
            Some(FetchLevel::Detailed) => Ok(Self::Detailed(
                DetailedShop::from_table(pool, shop, descendant_fetch_level).await?,
            )),
            None => Ok(Self::Default(shop.into())),
        }
    }

    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        ids: sqlx::types::Uuid,
        fetch_level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let shop = ShopTable::get_by_id(pool, ids).await?;

        Self::from_table(pool, shop, fetch_level, descendant_fetch_level).await
    }

    pub async fn query(
        pool: &sqlx::PgPool,
        filter: &Option<FilterConfig<QueryableShop>>,
        sorting: &Option<SortingConfig<SortableShop>>,
        pagination: &Option<PaginationConfig>,
        level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let shops = db::ShopTable::query(pool, filter, sorting, pagination).await?;

        let mut result = vec![];
        for shop in shops {
            let data = Self::from_table(pool, shop, level, descendant_fetch_level).await?;
            result.push(data);
        }
        Ok(result)
    }
}

impl Serialize for Shop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Shop::IdOnly(shop) => shop.serialize(serializer),
            Shop::Compact(shop) => shop.serialize(serializer),
            Shop::Default(shop) => shop.serialize(serializer),
            Shop::Detailed(shop) => shop.serialize(serializer),
        }
    }
}
