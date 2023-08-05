use mysk_lib::models::common::requests::FetchLevel;
use serde::{Deserialize, Serialize};
use sqlx::pool;

pub(crate) mod db;
pub(crate) mod fetch_levels;

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
                fetch_levels::compact::CompactOrder::from_table(
                    pool,
                    order,
                    descendant_fetch_level,
                )
                .await?,
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
