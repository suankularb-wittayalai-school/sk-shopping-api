use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FilterConfig, PaginationConfig, SortingConfig};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::common::RangeQuery;

use super::request::{QueryableListing, SortableListing};

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ListingTable {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub shop_id: sqlx::types::Uuid,
    pub is_hidden: bool,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub lifetime_stock: Option<i64>,
    pub amount_sold: Option<i64>,
}

impl ListingTable {
    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
    ) -> Result<Self, sqlx::Error> {
        let query = format!(
            "{} WHERE listings.id = $1 GROUP BY listings.id",
            Self::get_default_query()
        );

        let result = sqlx::query_as::<_, Self>(&query)
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(
            "{} WHERE listings.id = ANY($1) GROUP BY listings.id",
            Self::get_default_query()
        );

        let result = sqlx::query_as::<_, Self>(&query)
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT listings.*, MIN(price) as price, MIN(discounted_price) as discounted_price, CAST(SUM(stock_added) as INT8) as lifetime_stock, CAST(SUM(amount) as INT8) as amount_sold, MIN(preorder_start) as preorder_start, MAX(preorder_end) as preorder_end
        FROM listings 
        INNER JOIN items ON listings.id = items.listing_id 
        LEFT JOIN item_stock_updates ON item_stock_updates.item_id = items.id
        LEFT JOIN order_items ON order_items.item_id = items.id
        ".to_string()
    }

    fn get_count_query() -> String {
        "SELECT COUNT(*) FROM listings".to_string()
    }

    fn append_where_clause<'a>(
        query: &mut String,
        filter: &'a FilterConfig<QueryableListing>,
        params_count: i32,
    ) -> (
        i32,
        (
            Vec<String>,
            Vec<&'a sqlx::types::Uuid>,
            Vec<&'a Vec<sqlx::types::Uuid>>,
            Vec<bool>,
            Vec<&'a RangeQuery>,
        ),
    ) {
        let mut params_count = params_count;

        let mut string_params = Vec::new();
        let mut uuid_params = Vec::new();
        let mut uuid_array_params = Vec::new();
        let mut bool_params = Vec::new();
        let mut range_params = Vec::new();

        if let Some(q) = &filter.q {
            if query.contains("WHERE") {
                query.push_str(&format!(
                    " AND (listing.name ILIKE ${} OR description ILIKE ${})",
                    params_count + 1,
                    params_count + 1
                ));
            } else {
                query.push_str(&format!(
                    " WHERE (listing.name ILIKE ${} OR description ILIKE ${})",
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
                    query.push_str(&format!(" AND listing.name ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE listing.name ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", name));
                params_count += 1;
            }

            if let Some(description) = &data.description {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND description ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE description ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", description));
                params_count += 1;
            }

            if let Some(id) = &data.id {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND listing.id = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE listing.id = ${}", params_count + 1));
                }

                uuid_params.push(id);
                params_count += 1;
            }

            if let Some(shop_ids) = &data.shop_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND shop_id = ANY(${})", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE shop_id = ANY(${})", params_count + 1));
                }

                uuid_array_params.push(shop_ids);
                params_count += 1;
            }

            if let Some(collection_ids) = &data.collection_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND listings.id = ANY(
                            SELECT listing_id FROM listing_collections WHERE collection_id = ANY(${})
                        )",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE listings.id = ANY(
                            SELECT listing_id FROM listing_collections WHERE collection_id = ANY(${})
                        )",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(collection_ids);
                params_count += 1;
            }

            if let Some(item_ids) = &data.item_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND listings.id = ANY(
                            SELECT listing_id FROM items WHERE id = ANY(${})
                        )",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE listings.id = ANY(
                            SELECT listing_id FROM items WHERE id = ANY(${})
                        )",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(item_ids);
                params_count += 1;
            }

            if let Some(category_ids) = &data.category_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND listings.id = ANY(
                            SELECT listing_id FROM listing_categories WHERE category_id = ANY(${})
                        )",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE listings.id = ANY(
                            SELECT listing_id FROM listing_categories WHERE category_id = ANY(${})
                        )",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(category_ids);
                params_count += 1;
            }

            if let Some(is_hidden) = data.is_hidden {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND is_hidden = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE is_hidden = ${}", params_count + 1));
                }

                bool_params.push(is_hidden);
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
            (
                string_params,
                uuid_params,
                uuid_array_params,
                bool_params,
                range_params,
            ),
        )
    }

    fn append_order_clause(query: &mut String, order: &Option<SortingConfig<SortableListing>>) {
        let order = match order {
            Some(order) => order,
            None => return,
        };

        let sort_vec = match order.by.is_empty() {
            true => vec![SortableListing::Id],
            false => order.by.clone(),
        };

        if !sort_vec.is_empty() {
            query.push_str(" ORDER BY ");

            let mut first = true;
            for s in sort_vec {
                if !first {
                    query.push_str(", ");
                }

                match s {
                    SortableListing::Id => query.push_str("id"),
                    SortableListing::Name => query.push_str("name"),
                    SortableListing::CreatedAt => query.push_str("created_at"),
                    SortableListing::Price => query.push_str("price"),
                    SortableListing::Stock => query.push_str(" lifetime_stock"),
                }

                first = false;
            }

            match order.ascending {
                Some(true) => query.push_str(" ASC"),
                Some(false) => query.push_str(" DESC"),
                None => query.push_str(" ASC"),
            }
        }
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
        filter: &Option<FilterConfig<QueryableListing>>,
        sorting: &Option<SortingConfig<SortableListing>>,
        pagination: &Option<PaginationConfig>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = Self::get_default_query();

        let (
            params_count,
            (string_params, uuid_params, uuid_array_params, bool_params, range_params),
        ) = if let Some(filter) = filter {
            Self::append_where_clause(&mut query, filter, 0)
        } else {
            (
                0,
                (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            )
        };

        // add group by clause
        query.push_str(" GROUP BY listings.id");

        Self::append_order_clause(&mut query, sorting);

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

        for param in bool_params {
            query_builder = query_builder.bind(param);
        }

        for param in range_params {
            query_builder = query_builder.bind(param.min);
            query_builder = query_builder.bind(param.max);
        }

        for param in pagination_params {
            query_builder = query_builder.bind(param as i32);
        }

        query_builder.fetch_all(pool).await
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        let query = format!("DELETE FROM listings WHERE id = $1 ");

        match sqlx::query(&query).bind(id).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn bulk_delete(pool: &sqlx::PgPool, ids: Vec<Uuid>) -> Result<(), sqlx::Error> {
        let query = format!("DELETE FROM listings WHERE id = ANY($1) ");

        match sqlx::query(&query).bind(ids).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
