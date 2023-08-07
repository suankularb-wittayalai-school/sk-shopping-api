use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::pool;
use uuid::Uuid;

use self::db::OrderItemTable;

use super::item::Item;

pub(crate) mod db;
pub(crate) mod fetch_levels;

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub item: Item,
    pub amount: i64,
}

impl OrderItem {
    pub async fn from_table(
        pool: &sqlx::PgPool,
        order_item: db::OrderItemTable,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let item = Item::get_by_id(
            pool,
            order_item.item_id,
            descendant_fetch_level,
            Some(&FetchLevel::IdOnly),
        )
        .await?;

        Ok(Self {
            id: order_item.id,
            item,
            amount: order_item.amount,
        })
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let order_items_db = OrderItemTable::get_by_ids(pool, ids).await?;

        let mut order_items = Vec::new();

        for order_item in order_items_db {
            order_items.push(Self::from_table(pool, order_item, descendant_fetch_level).await?);
        }

        Ok(order_items)
    }
}

#[derive(Debug, Deserialize)]
pub enum Order {
    Compact(fetch_levels::compact::CompactOrder),
    Default(fetch_levels::default::DefaultOrder),
    IdOnly(fetch_levels::id_only::IdOnlyOrder),
    Detailed(fetch_levels::default::DefaultOrder),
}

impl Order {
    pub async fn from_table(
        pool: &pool::Pool<sqlx::Postgres>,
        order: db::OrderTable,
        fetch_level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        match fetch_level {
            Some(FetchLevel::Compact) => Ok(Self::Compact(
                fetch_levels::compact::CompactOrder::from_table(pool, order).await?,
            )),
            Some(FetchLevel::Default) => Ok(Self::Default(
                fetch_levels::default::DefaultOrder::from_table(
                    pool,
                    order,
                    descendant_fetch_level,
                )
                .await?,
            )),
            Some(FetchLevel::IdOnly) => Ok(Self::IdOnly(fetch_levels::id_only::IdOnlyOrder::from(
                order,
            ))),
            Some(FetchLevel::Detailed) => Ok(Self::Detailed(
                fetch_levels::default::DefaultOrder::from_table(
                    pool,
                    order,
                    descendant_fetch_level,
                )
                .await?,
            )),
            None => Ok(Self::IdOnly(fetch_levels::id_only::IdOnlyOrder::from(
                order,
            ))),
        }
    }

    pub async fn get_by_id(
        pool: &pool::Pool<sqlx::Postgres>,
        id: Uuid,
        fetch_level: Option<&FetchLevel>,
        descendant_fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, sqlx::Error> {
        let order_db = db::OrderTable::get_by_id(pool, id).await?;

        Self::from_table(pool, order_db, fetch_level, descendant_fetch_level).await
    }
}

impl Serialize for Order {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Order::Compact(order) => order.serialize(serializer),
            Order::Default(order) => order.serialize(serializer),
            Order::IdOnly(order) => order.serialize(serializer),
            Order::Detailed(order) => order.serialize(serializer),
        }
    }
}
