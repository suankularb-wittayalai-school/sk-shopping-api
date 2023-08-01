use chrono::NaiveDateTime;
use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};

use super::shop::Shop;

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyItem {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactItem {
    pub id: String,
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
    pub id: String,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub preorder_start: Option<NaiveDateTime>,
    pub preorder_end: Option<NaiveDateTime>,
    pub colors: Vec<String>,
    pub images_url: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedItem {
    pub id: String,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub lifetime_stock: i64,
    pub amount_sold: i64,
    pub preorder_start: Option<NaiveDateTime>,
    pub preorder_end: Option<NaiveDateTime>,
    pub colors: Vec<String>,
    pub images_url: Vec<String>,
    pub description: String,
    pub shop: Shop,
    // TODO
    // pub listing: Listing,
    // pub collection: Collection,
}

impl From<db::ItemTable> for IdOnlyItem {
    fn from(item: db::ItemTable) -> Self {
        Self { id: item.id }
    }
}

impl CompactItem {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        item: db::ItemTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Self {
        // let colors = db::ItemColorTable::find_by_item_id(&item.id).await?;
        CompactItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            // TODO
            // lifetime_stock: item.lifetime_stock,
            // amount_sold: item.amount_sold,
            // colors: colors.into_iter().map(|c| c.color).collect(),
            lifetime_stock: 0,
            amount_sold: 0,
            colors: vec![],
        }
    }
}

impl DefaultItem {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        item: db::ItemTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Self {
        // let colors = db::ItemColorTable::find_by_item_id(&item.id).await?;
        DefaultItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            // TODO
            // lifetime_stock: item.lifetime_stock,
            // amount_sold: item.amount_sold,
            // preorder_start: item.preorder_start,
            // preorder_end: item.preorder_end,
            // colors: colors.into_iter().map(|c| c.color).collect(),
            lifetime_stock: 0,
            amount_sold: 0,
            preorder_start: None,
            preorder_end: None,
            colors: vec![],
            images_url: vec![],
        }
    }
}

impl DetailedItem {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        item: db::ItemTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Self {
        // let colors = db::ItemColorTable::find_by_item_id(&item.id).await?;
        DetailedItem {
            id: item.id,
            name: item.name,
            variant_name: item.variant_name,
            price: item.price,
            discounted_price: item.discounted_price,
            // TODO
            // lifetime_stock: item.lifetime_stock,
            // amount_sold: item.amount_sold,
            // preorder_start: item.preorder_start,
            // preorder_end: item.preorder_end,
            // colors: colors.into_iter().map(|c| c.color).collect(),
            // images_url: db::ItemImageTable::find_by_item_id(&item.id).await?,
            // shop: Shop::from_table(db::ShopTable::find_by_id(&item.shop_id).await?)?,
            lifetime_stock: 0,
            amount_sold: 0,
            preorder_start: None,
            preorder_end: None,
            colors: vec![],
            images_url: vec![],
            description: "".to_string(),
            shop: Shop::from_table(
                pool,
                super::shop::db::ShopTable {
                    id: "".to_string(),
                    created_at: None,
                    name_th: "".to_string(),
                    name_en: None,
                    logo_url: "".to_string(),
                    is_school_pickup_allowed: false,
                    pickup_location: None,
                    is_delivery_allowed: false,
                    accept_promptpay: false,
                    promptpay_number: None,
                    accept_cod: false,
                    accent_color: None,
                    background_color: None,
                },
                None,
                None,
            )
            .await,
        }
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
    ) -> Self {
        match level {
            Some(FetchLevel::IdOnly) => Item::IdOnly(IdOnlyItem::from(item)),
            Some(FetchLevel::Compact) => {
                Item::Compact(CompactItem::from_table(pool, item, descendant_fetch_level).await)
            }
            Some(FetchLevel::Default) => {
                Item::Default(DefaultItem::from_table(pool, item, descendant_fetch_level).await)
            }
            Some(FetchLevel::Detailed) => {
                Item::Detailed(DetailedItem::from_table(pool, item, descendant_fetch_level).await)
            }
            _ => Item::IdOnly(IdOnlyItem::from(item)),
        }
    }
}
