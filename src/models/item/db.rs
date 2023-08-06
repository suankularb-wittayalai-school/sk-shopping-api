use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FilterConfig, PaginationConfig};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::common::RangeQuery;

use super::request::QueryableItem;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ItemTable {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name: String,
    pub variant_name: Option<String>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub listing_id: sqlx::types::Uuid,
}

impl ItemTable {
    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
    ) -> Result<Self, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>("SELECT * FROM items WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>("SELECT * FROM items WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT * FROM items".to_string()
    }

    fn get_count_query() -> String {
        "SELECT COUNT(*) FROM items".to_string()
    }

    fn append_where_clause<'a>(
        query: &mut String,
        filter: &'a FilterConfig<QueryableItem>,
        params_count: i32,
    ) -> (
        i32,
        (
            Vec<String>,
            Vec<&'a sqlx::types::Uuid>,
            Vec<&'a Vec<sqlx::types::Uuid>>,
            Vec<&'a RangeQuery>,
        ),
    ) {
        let mut params_count = params_count;

        let mut string_params = Vec::new();
        let mut uuid_params = Vec::new();
        let mut uuid_array_params = Vec::new();
        let mut range_params = Vec::new();

        if let Some(q) = &filter.q {
            if query.contains("WHERE") {
                query.push_str(&format!(
                    " AND (name ILIKE ${} OR variant_name ILIKE ${})",
                    params_count + 1,
                    params_count + 1
                ));
            } else {
                query.push_str(&format!(
                    " WHERE (name ILIKE ${} OR variant_name ILIKE ${})",
                    params_count + 1,
                    params_count + 1
                ));
            }

            string_params.push(format!("%{}%", q));
            params_count += 1;
        }

        if let Some(data) = &filter.data {
            if let Some(name) = &data.name {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND name ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE name ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", name));
                params_count += 1;
            }

            if let Some(id) = &data.id {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id = ${}", params_count + 1));
                }

                uuid_params.push(id);
                params_count += 1;
            }

            if let Some(shop_ids) = &data.shop_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND listing_id IN (SELECT id FROM listings WHERE shop_id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE listing_id IN (SELECT id FROM listings WHERE shop_id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(shop_ids);
                params_count += 1;
            }

            if let Some(collection_ids) = &data.collection_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND listing_id IN (SELECT listing_id FROM collection_listings WHERE collection_id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE listing_id IN (SELECT listing_id FROM collection_listings WHERE collection_id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(collection_ids);
                params_count += 1;
            }

            if let Some(listing_ids) = &data.listing_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND listing_id = ANY(${})", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE listing_id = ANY(${})", params_count + 1));
                }

                uuid_array_params.push(listing_ids);
                params_count += 1;
            }

            if let Some(price_range) = &data.price_range {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND (price >= ${} AND price <= ${})",
                        params_count + 1,
                        params_count + 2
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE (price >= ${} AND price <= ${})",
                        params_count + 1,
                        params_count + 2
                    ));
                }

                range_params.push(price_range);
                params_count += 1;
            }

            if let Some(stock_range) = &data.stock_range {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND (stock >= ${} AND stock <= ${})",
                        params_count + 1,
                        params_count + 2
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE (stock >= ${} AND stock <= ${})",
                        params_count + 1,
                        params_count + 2
                    ));
                }

                range_params.push(stock_range);
                params_count += 1;
            }
        }

        (
            params_count,
            (string_params, uuid_params, uuid_array_params, range_params),
        )
    }

    fn append_limit_clause(
        query: &mut String,
        pagination: &Option<PaginationConfig>,
        params_count: i32,
    ) -> (i32, Vec<u32>) {
        let mut params_count = params_count;
        let mut params = Vec::new();

        let pagination = match pagination {
            Some(pagination) => pagination,
            None => &PaginationConfig {
                p: 0,
                size: Some(50),
            },
        };

        if let Some(size) = pagination.size {
            query.push_str(&format!(" LIMIT ${}", params_count + 1));
            params.push(size);
            params_count += 1;
        }

        query.push_str(&format!(" OFFSET ${}", params_count + 1));
        params.push(pagination.p);
        params_count += 1;

        (params_count, params)
    }

    pub async fn query(
        pool: &sqlx::PgPool,
        filter: &Option<FilterConfig<QueryableItem>>,
        // sorting: &Option<SortingConfig<Sortable>>,
        pagination: &Option<PaginationConfig>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = Self::get_default_query();

        let (params_count, (string_params, uuid_params, uuid_array_params, range_params)) =
            if let Some(filter) = filter {
                Self::append_where_clause(&mut query, filter, 0)
            } else {
                (0, (Vec::new(), Vec::new(), Vec::new(), Vec::new()))
            };

        dbg!(query.clone());
        dbg!(&string_params);

        // Sorting

        // Pagination
        let (_params_count, pagination_params) =
            Self::append_limit_clause(&mut query, pagination, params_count);

        let mut query_builder = sqlx::query_as::<_, Self>(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        for param in uuid_params {
            query_builder = query_builder.bind(param);
        }

        for param in uuid_array_params {
            query_builder = query_builder.bind(param);
        }

        for param in range_params {
            query_builder = query_builder.bind(param.min);
            query_builder = query_builder.bind(param.max);
        }

        for param in pagination_params {
            let param = param as i32;
            query_builder = query_builder.bind(param);
        }

        query_builder.fetch_all(pool).await
    }
}
