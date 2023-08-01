use std::vec;

use mysk_lib::models::common::{requests::FetchLevel, string::MultiLangString};
use serde::{Deserialize, Serialize};

use super::{collection::Collection, item::Item, listing::Listing};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyShop {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactShop {
    pub id: String,
    pub name: MultiLangString,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultShop {
    pub id: String,
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
    pub id: String,
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
            logo_url: Some(shop.logo_url),
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
            logo_url: Some(shop.logo_url),
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
    pub async fn from_table(
        pool: &sqlx::PgPool,
        shop: db::ShopTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: shop.id,
            name: MultiLangString::new(shop.name_en, shop.name_th),
            accent_color: shop.accent_color,
            background_color: shop.background_color,
            logo_url: Some(shop.logo_url),
            is_school_pickup_allowed: shop.is_school_pickup_allowed,
            pickup_location: shop.pickup_location,
            is_delivery_allowed: shop.is_delivery_allowed,
            accept_promptpay: shop.accept_promptpay,
            promptpay_number: shop.promptpay_number,
            accept_cod: shop.accept_cod,
            // TODO: get listings and items from db
            items: vec![],
            listings: vec![],
            collections: vec![],
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
