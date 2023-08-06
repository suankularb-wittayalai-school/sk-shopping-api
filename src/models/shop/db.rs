use chrono::{DateTime, Utc};
use mysk_lib::models::common::requests::{FilterConfig, PaginationConfig, SortingConfig};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::request::{QueryableShop, SortableShop};

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ShopTable {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name_th: String,
    pub name_en: Option<String>,
    pub logo_url: Option<String>,
    pub is_school_pickup_allowed: bool,
    pub pickup_location: Option<String>,
    pub pickup_description: Option<String>,
    pub is_delivery_allowed: bool,
    pub accept_promptpay: bool,
    pub promptpay_number: Option<String>,
    pub accept_cod: bool,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
}

impl ShopTable {
    pub async fn get_by_id(
        pool: &sqlx::PgPool,
        id: sqlx::types::Uuid,
    ) -> Result<Self, sqlx::Error> {
        let result = sqlx::query_as::<_, Self>("SELECT * FROM shops WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    fn get_default_query() -> String {
        "SELECT * FROM shops".to_string()
    }

    fn append_where_clause(
        query: &mut String,
        filter: &FilterConfig<QueryableShop>,
        params_count: i32,
    ) -> (
        i32,
        (
            Vec<String>,
            Vec<sqlx::types::Uuid>,
            Vec<Vec<sqlx::types::Uuid>>,
            Vec<bool>,
        ),
    ) {
        let mut params_count = params_count;

        let mut string_params = Vec::new();
        let mut uuid_params = Vec::new();
        let mut uuid_array_params = Vec::new();
        let mut bool_params = Vec::new();

        if let Some(q) = &filter.q {
            if query.contains("WHERE") {
                query.push_str(&format!(
                    " AND (name_en ILIKE ${} OR name_th ILIKE ${})",
                    params_count + 1,
                    params_count + 1
                ));
            } else {
                query.push_str(&format!(
                    " WHERE (name_en ILIKE ${} OR name_th ILIKE ${})",
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

            if let Some(id) = data.id {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND id = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(" WHERE id = ${}", params_count + 1));
                }

                uuid_params.push(id);
                params_count += 1;
            }

            if let Some(collection_ids) = &data.collection_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND shop_id IN (SELECT shop_id FROM collections WHERE id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE shop_id IN (SELECT shop_id FROM collections WHERE id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(collection_ids.clone());
                params_count += 1;
            }

            if let Some(listing_ids) = &data.listing_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND shop_id IN (SELECT shop_id FROM listings WHERE id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE shop_id IN (SELECT shop_id FROM listings WHERE id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(listing_ids.clone());
                params_count += 1;
            }

            if let Some(item_ids) = &data.item_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND shop_id IN (SELECT shop_id FROM listings WHERE id IN (SELECT listing_id FROM items WHERE id = ANY(${})))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE shop_id IN (SELECT shop_id FROM listings WHERE id IN (SELECT listing_id FROM items WHERE id = ANY(${})))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(item_ids.clone());
                params_count += 1;
            }

            if let Some(manager_ids) = &data.manager_ids {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND shop_id IN (SELECT shop_id FROM shop_managers WHERE user_id = ANY(${}))",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE shop_id IN (SELECT shop_id FROM shop_managers WHERE user_id = ANY(${}))",
                        params_count + 1
                    ));
                }

                uuid_array_params.push(manager_ids.clone());
                params_count += 1;
            }

            if let Some(is_school_pickup_allowed) = data.is_school_pickup_allowed {
                if query.contains("WHERE") {
                    query.push_str(&format!(
                        " AND is_school_pickup_allowed = ${}",
                        params_count + 1
                    ));
                } else {
                    query.push_str(&format!(
                        " WHERE is_school_pickup_allowed = ${}",
                        params_count + 1
                    ));
                }

                bool_params.push(is_school_pickup_allowed);
                params_count += 1;
            }

            if let Some(is_delivery_allowed) = data.is_delivery_allowed {
                if query.contains("WHERE") {
                    query.push_str(&format!(" AND is_delivery_allowed = ${}", params_count + 1));
                } else {
                    query.push_str(&format!(
                        " WHERE is_delivery_allowed = ${}",
                        params_count + 1
                    ));
                }

                bool_params.push(is_delivery_allowed);
                params_count += 1;
            }
        }

        (
            params_count,
            (string_params, uuid_params, uuid_array_params, bool_params),
        )
    }

    fn append_order_clause(query: &mut String, shop: &Option<SortingConfig<SortableShop>>) {
        let shop = match shop {
            Some(shop) => shop,
            None => return,
        };

        let sort_vec = match shop.by.is_empty() {
            true => vec![SortableShop::Id],
            false => shop.by.clone(),
        };

        if !sort_vec.is_empty() {
            query.push_str(" ORDER BY ");

            let mut first = true;
            for s in sort_vec {
                if !first {
                    query.push_str(", ");
                }

                match s {
                    SortableShop::Id => query.push_str("id"),
                    SortableShop::NameEn => query.push_str("name_en"),
                    SortableShop::NameTh => query.push_str("name_ar"),
                    SortableShop::CreatedAt => query.push_str("variant_name"),
                    // SortableItem::Stock => query.push_str(" stock"),
                }

                first = false;
            }

            match shop.ascending {
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
        filter: &Option<FilterConfig<QueryableShop>>,
        sorting: &Option<SortingConfig<SortableShop>>,
        pagination: &Option<PaginationConfig>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut query = Self::get_default_query();

        let (params_count, (string_params, uuid_params, uuid_array_params, range_params)) =
            if let Some(filter) = filter {
                Self::append_where_clause(&mut query, filter, 0)
            } else {
                (0, (Vec::new(), Vec::new(), Vec::new(), Vec::new()))
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

        for param in range_params {
            query_builder = query_builder.bind(param);
            query_builder = query_builder.bind(param);
        }

        for param in pagination_params {
            let param = param as i32;
            query_builder = query_builder.bind(param);
        }

        query_builder.fetch_all(pool).await
    }
}
