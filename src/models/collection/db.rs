use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FilterConfig, PaginationConfig, SortingConfig};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::request::{QueryableCollection, SortableCollection};

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct CollectionTable {
    pub id: uuid::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: String,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
    pub shop_id: sqlx::types::Uuid,
}

impl CollectionTable {
    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
    ) -> Result<Self, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>("SELECT * FROM collections WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn get_by_ids(
        pool: &sqlx::PgPool,
        ids: Vec<sqlx::types::Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>("SELECT * FROM collections WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT * FROM collections".to_string()
    }

    fn get_count_query() -> String {
        "SELECT COUNT(*) FROM collections".to_string()
    }

    fn append_where_clause<'a>(
        query: &mut String,
        filter: &'a FilterConfig<QueryableCollection>,
        params_count: i32,
    ) -> (
        i32,
        (
            Vec<String>,
            Vec<sqlx::types::Uuid>,
            Vec<&'a Vec<sqlx::types::Uuid>>,
        ),
    ) {
        let mut params_count = params_count;

        let mut string_params = Vec::new();
        let mut uuid_params = Vec::new();
        let mut uuid_array_params = Vec::new();

        if let Some(q) = &filter.q {
            if query.contains("WHERE") {
                query.push_str(&format!(
                    " AND (name ILIKE ${} OR description ILIKE ${})",
                    params_count + 1,
                    params_count + 1
                ));
            } else {
                query.push_str(&format!(
                    " WHERE (name ILIKE ${} OR description ILIKE ${})",
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

            if let Some(description) = &data.description {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND description ILIKE ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE description ILIKE ${}", params_count + 1));
                }

                string_params.push(format!("%{}%", description));
                params_count += 1;
            }

            if let Some(id) = data.id {
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
                    query.push_str(&format!(" AND shop_id = ANY(${})", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE shop_id = ANY(${})", params_count + 1));
                }

                uuid_array_params.push(shop_ids);
                params_count += 1;
            }

            if let Some(listing_ids) = &data.listing_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id IN (SELECT collection_id FROM collection_listings WHERE listing_id = ANY(${}))", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id IN (SELECT collection_id FROM collection_listings WHERE listing_id = ANY(${}))", params_count + 1));
                }

                uuid_array_params.push(listing_ids);
                params_count += 1;
            }

            if let Some(item_ids) = &data.item_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id IN (SELECT collection_id FROM collection_listings INNER JOIN items ON items.listing_id = collection_listings.listing_id  WHERE item_id = ANY(${}))", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id IN (SELECT collection_id FROM collection_listings INNER JOIN items ON items.listing_id = collection_listings.listing_id  WHERE item_id = ANY(${}))", params_count + 1));
                }

                uuid_array_params.push(item_ids);
                params_count += 1;
            }
        }

        (
            params_count,
            (string_params, uuid_params, uuid_array_params),
        )
    }

    fn append_order_clause(query: &mut String, order: &Option<SortingConfig<SortableCollection>>) {
        let order = match order {
            Some(order) => order,
            None => return,
        };

        let sort_vec = match order.by.is_empty() {
            true => vec![SortableCollection::Id],
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
                    SortableCollection::Id => query.push_str("id"),
                    SortableCollection::Name => query.push_str("name"),
                    SortableCollection::CreatedAt => query.push_str("created_at"),
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
        filter: &Option<FilterConfig<QueryableCollection>>,
        sorting: &Option<SortingConfig<SortableCollection>>,
        pagination: &Option<PaginationConfig>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = Self::get_default_query();

        let (params_count, (string_params, uuid_params, uuid_array_params)) =
            if let Some(filter) = filter {
                Self::append_where_clause(&mut query, filter, 0)
            } else {
                (0, (Vec::new(), Vec::new(), Vec::new()))
            };

        // dbg!(query.clone());
        // dbg!(&string_params);

        // Sorting
        Self::append_order_clause(&mut query, sorting);

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

        for param in pagination_params {
            let param = param as i32;
            query_builder = query_builder.bind(param);
        }

        query_builder.fetch_all(pool).await
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        let query = format!("DELETE FROM collections WHERE id = $1 ");

        match sqlx::query(&query).bind(id).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn bulk_delete(pool: &sqlx::PgPool, ids: Vec<Uuid>) -> Result<(), sqlx::Error> {
        let query = format!("DELETE FROM collections WHERE id = ANY($1) ");

        match sqlx::query(&query).bind(ids).execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
